use std::time::Duration;

use aleph_client::{
    substrate_api_client::{extrinsic::balances::BalanceTransferXt, GenericAddress},
    Connection, KeyPair, MultisigParty,
};
use anyhow::Result as AnyResult;
use rand::{seq::index::sample, thread_rng, Rng};
use serde::Deserialize;

use chain_support::keypair_derived_from_seed;
use common::{Ident, Scenario, ScenarioLogging};
use scenarios_support::parse_interval;

use crate::{Action, Cancel, PartySize, Strategy, Threshold};

/// We operate on an account pool based on this seed. The final seeds will have
/// a form of `MULTISIG_SEED{i: usize}`.
const MULTISIG_SEED: &str = "//Multisig";

/// We expect that there are as many endowed accounts (of seed phrases: `MULTISIG_SEED{i}`,
/// where `i` is from 0 to this value (exclusively)).
const AVAILABLE_ACCOUNTS: usize = 50;

/// Returns keypair of the common account with index `idx`.
fn compute_keypair(idx: usize) -> KeyPair {
    keypair_derived_from_seed(format!("{}{}", MULTISIG_SEED, idx))
}

type Call = BalanceTransferXt;

/// Configuration for `Multisig` scenario.
#[derive(Clone, Debug, Deserialize)]
pub struct MultisigConfig {
    /// Unique string identifier for the scenario.
    ident: Ident,
    /// Periodicity of launching.
    #[serde(deserialize_with = "parse_interval")]
    interval: Duration,
    /// Multisig party will be derived from `party_size` and `threshold`. Each time
    /// scenario is launched, these parameters will potentially be different.
    party_size: PartySize,
    threshold: Threshold,
    /// How to conduct aggregation.
    strategy: Strategy,
    /// Whether after some time we should cancel the aggregation.
    cancel: bool,
}

/// Scenario performing multisignature aggregation.
#[derive(Clone)]
pub struct Multisig {
    ident: Ident,
    interval: Duration,
    party_size: PartySize,
    threshold: Threshold,
    strategy: Strategy,
    cancel: bool,
    connection: Connection,
}

impl Multisig {
    pub fn new(connection: &Connection, config: MultisigConfig) -> Self {
        Multisig {
            ident: config.ident,
            interval: config.interval,
            party_size: config.party_size,
            threshold: config.threshold,
            strategy: config.strategy,
            cancel: config.cancel,
            connection: connection.clone(),
        }
    }

    /// Randomly selects `party_size` accounts.
    fn select_members(&self, party_size: usize) -> Vec<KeyPair> {
        let mut rng = thread_rng();
        sample(&mut rng, AVAILABLE_ACCOUNTS, party_size)
            .iter()
            .map(compute_keypair)
            .collect()
    }

    /// Returns a sequence of actions. There will be either
    /// - `threshold` actions starting with an initiating one and `threshold - 1`
    ///   further approvals
    /// - or one initiating action and `< threshold - 1` other approvals with `Cancel`
    ///   action at the end; this case is true iff `self.cancel`
    ///
    /// The precise form of actions (with call or with hash only) depends on `self.strategy`.
    fn prepare_actions(&self, threshold: usize) -> Vec<Action> {
        let mut actions = vec![self.strategy.initial_action()];
        let mut call_submitted = actions[0].requires_call();

        for _ in 1..(threshold - 1) {
            let next_action = self.strategy.middle_action();
            call_submitted |= next_action.requires_call();
            actions.push(next_action)
        }
        actions.push(self.strategy.final_action(call_submitted));

        if self.cancel {
            let i = thread_rng().gen_range(1..threshold);
            actions[i] = Cancel;
            actions.truncate(i + 1)
        }

        actions
    }

    /// Dummy extrinsic to be executed after reaching threshold. We use simple money
    /// transfer which will always fail, but this does not matter at all in context
    /// of scenario success.
    fn prepare_call(&self) -> Call {
        self.connection
            .balance_transfer(GenericAddress::Address32(Default::default()), 0)
    }

    /// Due to the problems described in `crate::Action::perform` we have to create
    /// party for each call. Fortunately it is not so expensive.
    fn get_party(members: &[KeyPair], threshold: usize) -> AnyResult<MultisigParty> {
        MultisigParty::new(members.to_vec(), threshold as u16)
    }

    /// Executes `actions`. `i`th action will be performed by `members[i]` (unless
    /// this is `Cancel` which should be performed by `members[0]`.
    async fn perform_multisig(
        &self,
        members: Vec<KeyPair>,
        threshold: usize,
        actions: Vec<Action>,
        call: Call,
    ) -> AnyResult<()> {
        let connection = self.connection.clone().set_signer(members[0].clone());

        let party = Self::get_party(&members, threshold)?;
        self.info("Initializing signature aggregation");
        let mut sig_agg = actions[0]
            .perform(&connection, party, None, call.clone(), &members[0], false)
            .await?;

        // Here `i` is one less then the actual member index.
        for (i, action) in actions[1..].iter().enumerate() {
            let should_finalize = i + 2 == actions.len();

            self.info(format!(
                "Performing `{:?}`. Should finalize: {}",
                action, should_finalize
            ));

            let idx = if action.is_cancel() { 0 } else { i + 1 };

            sig_agg = action
                .perform(
                    &connection,
                    Self::get_party(&members, threshold)?,
                    sig_agg,
                    call.clone(),
                    &members[idx],
                    should_finalize,
                )
                .await?;

            if action.is_cancel() {
                break;
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Scenario for Multisig {
    fn interval(&self) -> Duration {
        self.interval
    }

    async fn play(&mut self) -> AnyResult<()> {
        let party_size = self.party_size.clone().get(AVAILABLE_ACCOUNTS)?;
        let threshold = self.threshold.clone().get(party_size)?;

        self.info(format!(
            "Starting multisig scenario with party size: {} and threshold: {}",
            party_size, threshold
        ));

        let members = self.select_members(party_size);
        let actions = self.prepare_actions(threshold);
        let call = self.prepare_call();

        let result = self
            .perform_multisig(members, threshold, actions, call)
            .await;
        self.handle(result)?;

        self.info("Scenario finished successfully");
        Ok(())
    }

    fn ident(&self) -> Ident {
        self.ident.clone()
    }
}
