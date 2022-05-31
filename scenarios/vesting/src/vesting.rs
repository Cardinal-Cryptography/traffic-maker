use crate::events::VestingUpdated;
use aleph_client::{
    account_from_keypair, substrate_api_client::AccountId, vest, vest_other, vested_transfer,
    AnyConnection, Connection, KeyPair, SignedConnection, VestingSchedule,
};
use anyhow::Result as AnyResult;
use chain_support::{do_async, keypair_derived_from_seed, with_event_matching};
use common::{Ident, Scenario, ScenarioLogging};
use rand::random;
use scenarios_support::parse_interval;
use serde::Deserialize;
use std::time::Duration;
use thiserror::Error;
use tokio::{task::spawn_blocking, time::sleep};

const SOURCE_VEST_SEED: &str = "//Vest/Source/Vest";
const SOURCE_VEST_OTHER_SEED: &str = "//Vest/Source/VestOther";
const RECIPIENT_SEED: &str = "//Vest/Recipient";
const INITIAL_VESTED: u128 = 1_000_000_000;
const PER_BLOCK: u128 = 1_000_000;
const WAIT_BLOCKS: u64 = 20;
const WAIT_PERIOD: Duration = Duration::from_secs(WAIT_BLOCKS);

#[derive(Clone, Copy, Debug, Deserialize)]
pub enum VestKind {
    Vest,
    VestOther,
}

#[derive(Debug, Error)]
pub enum VestError {
    #[error("🦺❌ The height of the current block could not be retrieved from storage.")]
    NoCurrentBlock,
}

fn random_recipient() -> KeyPair {
    keypair_derived_from_seed(format!("{}/{}", RECIPIENT_SEED, random::<u128>()).as_str())
}

/// Configuration for the [Vest] scenario.
#[derive(Clone, Debug, Deserialize)]
pub struct VestConfig {
    ident: Ident,
    #[serde(deserialize_with = "parse_interval")]
    interval: Duration,
    vest_kind: VestKind,
}

/// A scenario that goes through the vesting process.
///
/// The scenario sets up a new account, sending it some funds via a `vested_transfer`. Then it waits
/// some time (currently 20 seconds), and verifies that some of the funds have been unlocked.
/// Depending on `vest_kind` it either uses `vest` signed by the recipient of the `vested_transfer`
/// or `vest_other` signed by the sender of the `vested_transfer`.
pub struct Vest {
    ident: Ident,
    interval: Duration,
    connection: Connection,
    vest_kind: VestKind,
}

impl Vest {
    pub fn new<C: AnyConnection>(connection: &C, config: &VestConfig) -> Self {
        Vest {
            ident: config.ident.clone(),
            interval: config.interval,
            connection: connection.as_connection(),
            vest_kind: config.vest_kind,
        }
    }

    fn source(&self) -> KeyPair {
        match self.vest_kind {
            VestKind::Vest => keypair_derived_from_seed(SOURCE_VEST_SEED),
            VestKind::VestOther => keypair_derived_from_seed(SOURCE_VEST_OTHER_SEED),
        }
    }

    async fn current_block(&self) -> AnyResult<u32> {
        let connection = self.connection.clone();
        let result =
            spawn_blocking(move || connection.get_storage_value::<u32>("System", "Number", None))
                .await??
                .ok_or(VestError::NoCurrentBlock)?;
        Ok(result)
    }

    async fn vest_action(&self, target: &KeyPair) -> AnyResult<()> {
        match self.vest_kind {
            VestKind::Vest => self.vest(target).await,
            VestKind::VestOther => {
                self.vest_other(self.source(), &account_from_keypair(target))
                    .await
            }
        }
    }

    async fn vest_other(&self, signer: KeyPair, target: &AccountId) -> AnyResult<()> {
        let connection = SignedConnection::from_any_connection(&self.connection, signer);
        do_async!(vest_other, connection, target)?
    }

    async fn vest(&self, account: &KeyPair) -> AnyResult<()> {
        let connection = SignedConnection::from_any_connection(&self.connection, account.clone());
        do_async!(vest, connection)?
    }

    async fn vested_transfer(
        &self,
        receiver: &AccountId,
        schedule: VestingSchedule,
    ) -> AnyResult<()> {
        let connection = SignedConnection::from_any_connection(&self.connection, self.source());
        do_async!(vested_transfer, connection, receiver, schedule)?
    }

    async fn do_play(&self) -> AnyResult<()> {
        let current_block = self.current_block().await?;
        let recipient = random_recipient();
        let recipient_copy = recipient.clone();

        self.info("Setting up with vested_transfer");

        self.vested_transfer(
            &account_from_keypair(&recipient),
            VestingSchedule::new(INITIAL_VESTED, PER_BLOCK, current_block + 10),
        )
        .await?;

        self.info("Waiting for some of the funds to unlock");

        sleep(WAIT_PERIOD).await;

        self.info(format!("Calling {:?}", self.vest_kind));

        // There should be enough funds for the fee from what's already vested
        with_event_matching(
            &self.connection,
            move |event: &VestingUpdated| {
                let unlocked = INITIAL_VESTED - event.unvested;

                event.account == account_from_keypair(&recipient_copy)
                    && unlocked > PER_BLOCK * (WAIT_BLOCKS as u128) / 2
                    && unlocked < PER_BLOCK * (WAIT_BLOCKS as u128) * 5
            },
            Duration::from_secs(2),
            self.vest_action(&recipient),
        )
        .await?;

        self.info("Success");

        Ok(())
    }
}

#[async_trait::async_trait]
impl Scenario for Vest {
    fn interval(&self) -> Duration {
        self.interval
    }

    async fn play(&mut self) -> AnyResult<()> {
        self.handle(self.do_play().await)
    }

    fn ident(&self) -> Ident {
        self.ident.clone()
    }
}
