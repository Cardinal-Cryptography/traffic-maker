use std::time::Duration;

use aleph_client::{get_free_balance, Connection, KeyPair};
use serde::Deserialize;
use substrate_api_client::{AccountId, Pair};

use chain_support::{do_async, keypair_derived_from_seed, real_amount, try_transfer};
use common::{Ident, Scenario, ScenarioError, ScenarioLogging};

use crate::parse_interval;

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
pub struct RoundRobinProps {
    ident: Ident,
    #[serde(deserialize_with = "parse_interval")]
    interval: Duration,
    passes: usize,
    robin_value: u64,
}

impl RoundRobin {
    pub fn new(connection: &Connection, props: RoundRobinProps) -> Self {
        let accounts = (0..props.passes)
            .map(|i| keypair_derived_from_seed(&*format!("{}{}", ROUND_ROBIN_SEED, i)))
            .collect();
        RoundRobin {
            ident: props.ident,
            accounts,
            interval: props.interval,
            connection: connection.clone(),
            robin_value: real_amount(&props.robin_value),
        }
    }

    async fn pass_robin(&self, sender: KeyPair, receiver: AccountId) -> Result<(), ScenarioError> {
        let receiver_free_before: u128 = do_async!(get_free_balance, &self.connection, &receiver)?;

        let connection = self.connection.clone().set_signer(sender.clone());
        self.handle(do_async!(
            try_transfer,
            &connection,
            &receiver,
            self.robin_value
        )?)?;

        let receiver_free_after: u128 = do_async!(get_free_balance, &self.connection, &receiver)?;

        if receiver_free_after != receiver_free_before + self.robin_value {
            // It may happen that the balance is not as expected due to the
            // concurrent scenarios using this account.
            self.warn(&format!(
                "It doesn't seem like the robin has reached receiver. \
                Receiver's balance before: {} and after: {}. Robin value: {}",
                receiver_free_before, receiver_free_after, self.robin_value,
            ));
        } else {
            self.debug("Receiver has received the robin.");
        };

        Ok(())
    }
}

#[async_trait::async_trait]
impl Scenario for RoundRobin {
    fn interval(&self) -> Duration {
        self.interval
    }

    async fn play(&mut self) -> Result<(), ScenarioError> {
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
