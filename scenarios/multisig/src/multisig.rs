use std::time::Duration;

use aleph_client::{
    substrate_api_client::{extrinsic::balances::BalanceTransferXt, GenericAddress},
    Connection, KeyPair, MultisigParty,
};
use anyhow::Result as AnyResult;
use rand::{seq::index::sample, thread_rng, Rng};
use serde::Deserialize;

use crate::{Action, Cancel, PartySize, Strategy, Threshold};
use chain_support::keypair_derived_from_seed;
use common::{Ident, Scenario, ScenarioLogging};
use scenarios_support::parse_interval;

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

#[derive(Clone, Debug, Deserialize)]
pub struct MultisigConfig {
    ident: Ident,
    #[serde(deserialize_with = "parse_interval")]
    interval: Duration,
    party_size: PartySize,
    threshold: Threshold,
    strategy: Strategy,
    cancel: bool,
}

#[derive(Clone)]
pub struct Multisig {
    ident: Ident,
    interval: Duration,
    party_size: usize,
    threshold: usize,
    strategy: Strategy,
    cancel: bool,
    connection: Connection,
}

impl Multisig {
    pub fn new(connection: &Connection, config: MultisigConfig) -> Self {
        let party_size = config.party_size.get(AVAILABLE_ACCOUNTS);
        let threshold = config.threshold.get(party_size).unwrap();

        Multisig {
            ident: config.ident,
            interval: config.interval,
            party_size,
            threshold,
            strategy: config.strategy,
            cancel: config.cancel,
            connection: connection.clone(),
        }
    }

    fn select_members(&self) -> Vec<KeyPair> {
        let mut rng = thread_rng();
        sample(&mut rng, AVAILABLE_ACCOUNTS, self.party_size)
            .iter()
            .map(compute_keypair)
            .collect()
    }

    fn prepare_actions(&self) -> Vec<Action> {
        let mut actions = vec![self.strategy.initial_action()];
        let mut call_submitted = actions[0].requires_call();

        for _ in 1..(self.threshold - 1) {
            let next_action = self.strategy.middle_action();
            call_submitted |= next_action.requires_call();
            actions.push(next_action)
        }
        actions.push(self.strategy.final_action(call_submitted));

        if self.cancel {
            let i = thread_rng().gen_range(1..self.threshold);
            actions[i] = Cancel;
            actions.truncate(i + 1)
        }

        actions
    }

    fn prepare_call(&self) -> Call {
        self.connection
            .balance_transfer(GenericAddress::Address32(Default::default()), 0)
    }

    fn get_party(&self, members: &[KeyPair]) -> AnyResult<MultisigParty> {
        MultisigParty::new(members.to_vec(), self.threshold as u16)
    }

    async fn perform_multisig(
        &self,
        members: Vec<KeyPair>,
        actions: Vec<Action>,
        call: Call,
    ) -> AnyResult<()> {
        let connection = self.connection.clone().set_signer(members[0].clone());

        let party = self.get_party(&members)?;
        self.info("Initializing signature aggregation");
        let mut sig_agg = actions[0]
            .perform(&connection, party, None, call.clone(), &members[0], false)
            .await?;

        for (idx, action) in actions[1..].iter().enumerate() {
            let should_finalize = idx + 2 == actions.len();

            self.info(format!(
                "Performing `{:?}`. Should finalize: {}",
                action, should_finalize
            ));

            sig_agg = action
                .perform(
                    &connection,
                    self.get_party(&members)?,
                    sig_agg,
                    call.clone(),
                    &members[idx + 1],
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
        self.info(format!(
            "Starting multisig scenario with party size: {} and threshold: {}",
            self.party_size, self.threshold
        ));

        let members = self.select_members();
        let actions = self.prepare_actions();
        let call = self.prepare_call();

        let result = self.perform_multisig(members, actions, call).await;
        self.handle(result)?;

        self.info("Scenario finished successfully");
        Ok(())
    }

    fn ident(&self) -> Ident {
        self.ident.clone()
    }
}
