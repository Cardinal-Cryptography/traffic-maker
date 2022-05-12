use std::time::Duration;

use aleph_client::{account_from_keypair, substrate_api_client, Connection, KeyPair};
use anyhow::Result as AnyResult;
use rand::random;
use serde::Deserialize;
use substrate_api_client::AccountId;

use chain_support::{keypair_derived_from_seed, real_amount};
use common::{Ident, Scenario, ScenarioLogging};
use scenarios_support::parse_interval;

use crate::try_transfer;

/// This account should be included in the endowment list. The amount should be
/// proportional to the `transfer_value` props parameter.
const SENDER_SEED: &str = "//SimpleTransferSender";
const RECEIVER_SEED: &str = "//SimpleTransferReceiver";

#[derive(Clone, Debug, Deserialize)]
pub struct SimpleTransferConfig {
    ident: Ident,
    #[serde(deserialize_with = "parse_interval")]
    interval: Duration,
    transfer_value: u64,
}

#[derive(Clone)]
pub struct SimpleTransfer {
    ident: Ident,
    interval: Duration,
    sender: KeyPair,
    receiver: AccountId,
    connection: Connection,
    transfer_value: u128,
}

impl SimpleTransfer {
    pub fn new(connection: &Connection, config: SimpleTransferConfig) -> Self {
        let sender = keypair_derived_from_seed(SENDER_SEED);
        let connection = connection.clone().set_signer(sender.clone());

        let receiver = account_from_keypair(&keypair_derived_from_seed(RECEIVER_SEED));

        SimpleTransfer {
            ident: config.ident,
            interval: config.interval,
            sender,
            receiver,
            connection,
            transfer_value: real_amount(&config.transfer_value) + random::<u32>() as u128,
        }
    }
}

#[async_trait::async_trait]
impl Scenario for SimpleTransfer {
    fn interval(&self) -> Duration {
        self.interval
    }

    async fn play(&mut self) -> AnyResult<()> {
        self.info("Ready to go");

        let transfer_result = try_transfer(
            &self.connection,
            &self.sender,
            &self.receiver,
            self.transfer_value,
        )
        .await;
        self.handle(transfer_result)?;

        self.info("Done");
        Ok(())
    }

    fn ident(&self) -> Ident {
        self.ident.clone()
    }
}
