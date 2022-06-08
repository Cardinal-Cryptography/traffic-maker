use crate::events::VestingUpdated;
use aleph_client::{
    account_from_keypair, substrate_api_client::AccountId, vest, vest_other, vested_transfer,
    Connection, KeyPair, SignedConnection, VestingSchedule,
};
use anyhow::Result as AnyResult;
use chain_support::{do_async, keypair_derived_from_seed, with_event_matching};
use common::{Scenario, ScenarioLogging};
use rand::random;
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

/// A scenario that goes through the vesting process.
///
/// The scenario sets up a new account, sending it some funds via a `vested_transfer`. Then it waits
/// some time (currently 20 seconds), and verifies that some of the funds have been unlocked.
/// Depending on `vest_kind` it either uses `vest` signed by the recipient of the `vested_transfer`
/// or `vest_other` signed by the sender of the `vested_transfer`.
#[derive(Clone, Debug, Deserialize)]
pub struct Vest {
    vest_kind: VestKind,
}

impl Vest {
    fn source(&self) -> KeyPair {
        match self.vest_kind {
            VestKind::Vest => keypair_derived_from_seed(SOURCE_VEST_SEED),
            VestKind::VestOther => keypair_derived_from_seed(SOURCE_VEST_OTHER_SEED),
        }
    }

    async fn current_block(connection: &Connection) -> AnyResult<u32> {
        let connection = connection.clone();
        let result =
            spawn_blocking(move || connection.get_storage_value::<u32>("System", "Number", None))
                .await??
                .ok_or(VestError::NoCurrentBlock)?;
        Ok(result)
    }

    async fn vest_action(&self, connection: &Connection, target: &KeyPair) -> AnyResult<()> {
        match self.vest_kind {
            VestKind::Vest => Self::vest(connection, target).await,
            VestKind::VestOther => {
                Self::vest_other(connection, self.source(), &account_from_keypair(target)).await
            }
        }
    }

    async fn vest_other(
        connection: &Connection,
        signer: KeyPair,
        target: &AccountId,
    ) -> AnyResult<()> {
        let connection = SignedConnection::from_any_connection(connection, signer);
        do_async!(vest_other, connection, target)?
    }

    async fn vest(connection: &Connection, account: &KeyPair) -> AnyResult<()> {
        let connection = SignedConnection::from_any_connection(connection, account.clone());
        do_async!(vest, connection)?
    }

    async fn vested_transfer(
        &self,
        connection: &Connection,
        receiver: &AccountId,
        schedule: VestingSchedule,
    ) -> AnyResult<()> {
        let connection = SignedConnection::from_any_connection(connection, self.source());
        do_async!(vested_transfer, connection, receiver, schedule)?
    }

    async fn do_play(&self, connection: &Connection, logger: &ScenarioLogging) -> AnyResult<()> {
        let current_block = Self::current_block(connection).await?;
        let recipient = random_recipient();
        let recipient_copy = recipient.clone();

        logger.info("Setting up with vested_transfer");

        self.vested_transfer(
            connection,
            &account_from_keypair(&recipient),
            VestingSchedule::new(INITIAL_VESTED, PER_BLOCK, current_block + 10),
        )
        .await?;

        logger.info("Waiting for some of the funds to unlock");

        sleep(WAIT_PERIOD).await;

        logger.info(format!("Calling {:?}", self.vest_kind));

        // There should be enough funds for the fee from what's already vested
        with_event_matching(
            connection,
            move |event: &VestingUpdated| {
                let unlocked = INITIAL_VESTED - event.unvested;

                event.account == account_from_keypair(&recipient_copy)
                    && unlocked > PER_BLOCK * (WAIT_BLOCKS as u128) / 2
                    && unlocked < PER_BLOCK * (WAIT_BLOCKS as u128) * 5
            },
            Duration::from_secs(2),
            self.vest_action(connection, &recipient),
        )
        .await?;

        logger.info("Success");

        Ok(())
    }
}

#[async_trait::async_trait]
impl Scenario for Vest {
    async fn play(&mut self, connection: &Connection, logger: &ScenarioLogging) -> AnyResult<()> {
        logger.log_result(self.do_play(connection, logger).await)
    }
}
