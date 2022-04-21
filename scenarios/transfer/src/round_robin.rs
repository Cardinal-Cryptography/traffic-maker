use std::time::Duration;

use aleph_client::{Connection, KeyPair};
use anyhow::Result;
use chain_support::{keypair_derived_from_seed, real_amount};
use common::{Ident, Scenario, ScenarioLogging};
use rand::random;
use serde::Deserialize;
use substrate_api_client::{AccountId, Pair};

use crate::{parse_interval, try_transfer};

const ROUND_ROBIN_SEED: &str = "//RoundRobin";

#[derive(Clone)]
pub struct RoundRobin {
    ident: Ident,
    accounts: Vec<KeyPair>,
    interval: Duration,
    connection: Connection,
    robin_value: u128,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RoundRobinConfig {
    ident: Ident,
    #[serde(deserialize_with = "parse_interval")]
    interval: Duration,
    passes: usize,
    robin_value: u64,
}

impl RoundRobin {
    pub fn new(connection: &Connection, config: RoundRobinConfig) -> Self {
        let accounts = (0..config.passes)
            .map(|i| keypair_derived_from_seed(&*format!("{}{}", ROUND_ROBIN_SEED, i)))
            .collect();
        RoundRobin {
            ident: config.ident,
            accounts,
            interval: config.interval,
            connection: connection.clone(),
            robin_value: real_amount(&config.robin_value),
        }
    }

    async fn pass_robin(&self, sender: KeyPair, receiver: AccountId) -> Result<()> {
        let transfer_result = try_transfer(
            &self.connection,
            &sender,
            &receiver,
            self.robin_value + random::<u32>() as u128,
        )
        .await;

        self.handle(transfer_result)
    }
}

#[async_trait::async_trait]
impl Scenario for RoundRobin {
    fn interval(&self) -> Duration {
        self.interval
    }

    async fn play(&mut self) -> Result<()> {
        self.info("Starting scenario");

        let n = self.accounts.len();
        for sender_idx in 0..n {
            let receiver_idx = (sender_idx + 1) % n;
            let (sender, receiver) = (&self.accounts[sender_idx], &self.accounts[receiver_idx]);

            self.pass_robin(sender.clone(), AccountId::from(receiver.public()))
                .await?;

            self.debug(&*format!("Completed {}/{} passes.", sender_idx + 1, n));
        }

        self.info("Scenario finished successfully");
        Ok(())
    }

    fn ident(&self) -> Ident {
        self.ident.clone()
    }
}
