use crate::events::VestingUpdated;
use aleph_client::{
    account_from_keypair, substrate_api_client::AccountId, vest, vest_other, vested_transfer,
    Connection, KeyPair, VestingSchedule,
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

#[derive(Clone, Debug, Deserialize)]
pub struct VestConfig {
    ident: Ident,
    #[serde(deserialize_with = "parse_interval")]
    interval: Duration,
    vest_kind: VestKind,
}

pub struct Vest {
    ident: Ident,
    interval: Duration,
    connection: Connection,
    vest_kind: VestKind,
}

impl Vest {
    pub fn new(connection: &Connection, config: &VestConfig) -> AnyResult<Self> {
        Ok(Vest {
            ident: config.ident.clone(),
            interval: config.interval,
            connection: connection.clone(),
            vest_kind: config.vest_kind,
        })
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
        let connection = self.connection.clone().set_signer(signer);
        do_async!(vest_other, connection, target)?
    }

    async fn vest(&self, account: &KeyPair) -> AnyResult<()> {
        let connection = self.connection.clone().set_signer(account.clone());
        do_async!(vest, connection)?
    }

    async fn vested_transfer(
        &self,
        receiver: &AccountId,
        schedule: VestingSchedule,
    ) -> AnyResult<()> {
        let connection = self.connection.clone().set_signer(self.source());
        do_async!(vested_transfer, connection, receiver, schedule)?
    }

    async fn do_play(&self) -> AnyResult<()> {
        let current_block = self.current_block().await?;
        let recipient = random_recipient();
        let recipient_copy = recipient.clone();
        let initial_vested = 1000000000;
        let per_block = 1000000;

        self.info("Setting up with vested_transfer");

        self.vested_transfer(
            &account_from_keypair(&recipient),
            VestingSchedule::new(initial_vested, per_block, current_block + 10),
        )
        .await?;

        self.info("Waiting for some of the funds to unlock");

        sleep(Duration::from_secs(20)).await;

        self.info(format!("Calling {:?}", self.vest_kind));

        // There should be enough funds for the fee from what's already vested
        with_event_matching(
            &self.connection,
            move |event: &VestingUpdated| {
                let unlocked = initial_vested - event.unvested;

                event.account == account_from_keypair(&recipient_copy)
                    && unlocked > 10 * per_block
                    && unlocked < 100 * per_block
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
