use aleph_client::{account_from_keypair, substrate_api_client, Connection, KeyPair};
use anyhow::Result as AnyResult;
use rand::random;
use serde::Deserialize;
use substrate_api_client::AccountId;

use chain_support::{keypair_derived_from_seed, real_amount};
use common::{Scenario, ScenarioLogging};

use crate::try_transfer;

/// This account should be included in the endowment list. The amount should be
/// proportional to the `transfer_value` props parameter.
const SENDER_SEED: &str = "//SimpleTransferSender";
const RECEIVER_SEED: &str = "//SimpleTransferReceiver";

#[derive(Clone, Debug, Deserialize)]
pub struct SimpleTransfer {
    transfer_value: u64,
}

impl SimpleTransfer {
    fn sender() -> KeyPair {
        keypair_derived_from_seed(SENDER_SEED)
    }

    fn receiver() -> AccountId {
        account_from_keypair(&keypair_derived_from_seed(RECEIVER_SEED))
    }

    fn transfer_value(&self) -> u128 {
        real_amount(&self.transfer_value) + random::<u32>() as u128
    }
}

#[async_trait::async_trait]
impl Scenario for SimpleTransfer {
    async fn play(&mut self, connection: &Connection, logger: &ScenarioLogging) -> AnyResult<()> {
        logger.info("Ready to go");

        let transfer_result = try_transfer(
            connection,
            &Self::sender(),
            &Self::receiver(),
            self.transfer_value(),
        )
        .await;
        logger.handle(transfer_result)?;

        logger.info("Done");
        Ok(())
    }
}
