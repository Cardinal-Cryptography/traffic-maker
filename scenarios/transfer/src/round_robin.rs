use aleph_client::{account_from_keypair, substrate_api_client, Connection, KeyPair};
use anyhow::Result as AnyResult;
use rand::random;
use serde::Deserialize;
use substrate_api_client::AccountId;

use chain_support::{keypair_derived_from_seed, real_amount};
use common::{Scenario, ScenarioLogging};

use crate::try_transfer;

const ROUND_ROBIN_SEED: &str = "//RoundRobin";

#[derive(Clone, Debug, Deserialize)]
pub struct RoundRobin {
    passes: usize,
    robin_value: u64,
}

impl RoundRobin {
    fn account(id: usize) -> KeyPair {
        keypair_derived_from_seed(&*format!("{}{}", ROUND_ROBIN_SEED, id))
    }

    async fn pass_robin(
        &self,
        connection: &Connection,
        sender: KeyPair,
        receiver: AccountId,
        logger: &ScenarioLogging,
    ) -> AnyResult<()> {
        let transfer_result = try_transfer(
            connection,
            &sender,
            &receiver,
            real_amount(&self.robin_value) + random::<u32>() as u128,
        )
        .await;

        logger.log_result(transfer_result)
    }
}

#[async_trait::async_trait]
impl Scenario<Connection> for RoundRobin {
    async fn play(&mut self, connection: &Connection, logger: &ScenarioLogging) -> AnyResult<()> {
        logger.info("Starting scenario");

        for sender_idx in 0..self.passes {
            let receiver_idx = (sender_idx + 1) % self.passes;
            let sender = Self::account(sender_idx);
            let receiver = Self::account(receiver_idx);

            self.pass_robin(connection, sender, account_from_keypair(&receiver), logger)
                .await?;

            logger.debug(&*format!(
                "Completed {}/{} passes.",
                sender_idx + 1,
                self.passes
            ));
        }

        logger.info("Scenario finished successfully");
        Ok(())
    }
}
