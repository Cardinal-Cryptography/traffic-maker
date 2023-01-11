use std::time::Duration;

use aleph_client::{
    account_from_keypair, api, api::vesting::events::VestingUpdated,
    pallet_vesting::vesting_info::VestingInfo, pallets::vesting::VestingUserApi,
    utility::BlocksApi, AccountId, Connection, KeyPair, SignedConnection, TxStatus,
};
use anyhow::{anyhow, ensure, Result as AnyResult};
use chain_support::{keypair_derived_from_seed, Balance};
use common::{Scenario, ScenarioLogging};
use rand::random;
use serde::Deserialize;
use tokio::time::sleep;

const SOURCE_VEST_SEED: &str = "//Vest//Source//Vest";
const SOURCE_VEST_OTHER_SEED: &str = "//Vest//Source//VestOther";
const RECIPIENT_SEED: &str = "//Vest//Recipient";
const INITIAL_VESTED: Balance = 1_000_000_000;
const PER_BLOCK: Balance = 1_000_000;
const WAIT_BLOCKS: u64 = 20;
const WAIT_PERIOD: Duration = Duration::from_secs(WAIT_BLOCKS);

#[derive(Clone, Copy, Debug, Deserialize)]
pub enum VestKind {
    Vest,
    VestOther,
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

    async fn vest_action(
        &self,
        connection: &Connection,
        target: &KeyPair,
    ) -> AnyResult<VestingUpdated> {
        let submission = match self.vest_kind {
            VestKind::Vest => {
                connection
                    .as_client()
                    .tx()
                    .sign_and_submit_then_watch_default(&api::tx().vesting().vest(), target)
                    .await
            }
            VestKind::VestOther => {
                connection
                    .as_client()
                    .tx()
                    .sign_and_submit_then_watch_default(
                        &api::tx()
                            .vesting()
                            .vest_other(target.account_id().clone().into()),
                        &self.source(),
                    )
                    .await
            }
        };

        Ok(submission?
            .wait_for_finalized()
            .await?
            .fetch_events()
            .await?
            .find_first::<VestingUpdated>()?
            .unwrap())
    }

    async fn vested_transfer(
        &self,
        connection: &Connection,
        receiver: &AccountId,
        schedule: VestingInfo<Balance, u32>,
    ) -> AnyResult<()> {
        SignedConnection::from_connection(connection.clone(), self.source())
            .vested_transfer(receiver.clone(), schedule, TxStatus::Finalized)
            .await
            .map(|_| ())
    }

    async fn do_play(&self, connection: &Connection, logger: &ScenarioLogging) -> AnyResult<()> {
        let current_block = connection
            .get_best_block()
            .await?
            .ok_or(anyhow!("Failed to obtain best block info"))?;
        let recipient = random_recipient();

        logger.info("Setting up with vested_transfer");

        self.vested_transfer(
            connection,
            &account_from_keypair(recipient.signer()),
            VestingInfo {
                locked: INITIAL_VESTED,
                per_block: PER_BLOCK,
                starting_block: current_block + 10,
            },
        )
        .await?;

        logger.info("Waiting for some of the funds to unlock");

        sleep(WAIT_PERIOD).await;

        logger.info(format!("Calling {:?}", self.vest_kind));

        let VestingUpdated { account, unvested } = self.vest_action(connection, &recipient).await?;
        let unlocked = INITIAL_VESTED - unvested;
        ensure!(
            account == account_from_keypair(recipient.signer())
                && unlocked > PER_BLOCK * (WAIT_BLOCKS as Balance) / 2
                && unlocked < PER_BLOCK * (WAIT_BLOCKS as Balance) * 5,
            anyhow!("Incorrect event has been intercepted")
        );

        logger.info("Success");

        Ok(())
    }
}

#[async_trait::async_trait]
impl Scenario<Connection> for Vest {
    async fn play(&mut self, connection: &Connection, logger: &ScenarioLogging) -> AnyResult<()> {
        logger.log_result(self.do_play(connection, logger).await)
    }
}
