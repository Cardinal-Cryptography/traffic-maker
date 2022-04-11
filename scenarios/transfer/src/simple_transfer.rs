use std::time::Duration;

use aleph_client::{get_free_balance, Connection};
use serde::Deserialize;
use substrate_api_client::{AccountId, Pair};

use chain_support::{do_async, keypair_derived_from_seed, real_amount, try_transfer};
use common::{Ident, Scenario, ScenarioError, ScenarioLogging};

use crate::parse_interval;

/// This account should be included in the endowment list. The amount should be
/// proportional to the `transfer_value` props parameter.
const SENDER_SEED: &str = "//SimpleTransferSender";
const RECEIVER_SEED: &str = "//SimpleTransferReceiver";

#[derive(Clone, Debug, Deserialize)]
pub struct SimpleTransferProps {
    ident: Ident,
    #[serde(deserialize_with = "parse_interval")]
    interval: Duration,
    transfer_value: u64,
}

#[derive(Clone)]
pub struct SimpleTransfer {
    ident: Ident,
    interval: Duration,
    receiver: AccountId,
    connection: Connection,
    transfer_value: u128,
}

impl SimpleTransfer {
    pub fn new(connection: &Connection, props: SimpleTransferProps) -> Self {
        let sender = keypair_derived_from_seed(SENDER_SEED);
        let connection = connection.clone().set_signer(sender);

        let receiver = AccountId::from(keypair_derived_from_seed(RECEIVER_SEED).public());

        SimpleTransfer {
            ident: props.ident,
            interval: props.interval,
            receiver,
            connection,
            transfer_value: real_amount(&props.transfer_value),
        }
    }
}

#[async_trait::async_trait]
impl Scenario for SimpleTransfer {
    fn interval(&self) -> Duration {
        self.interval
    }

    async fn play(&mut self) -> Result<(), ScenarioError> {
        self.info("Ready to go");

        let receiver_balance_before: u128 =
            do_async!(get_free_balance, &self.connection, &self.receiver)?;

        self.handle(do_async!(
            try_transfer,
            &self.connection,
            &self.receiver,
            self.transfer_value
        )?)?;

        let receiver_balance_after: u128 =
            do_async!(get_free_balance, &self.connection, &self.receiver)?;

        if receiver_balance_after != receiver_balance_before + self.transfer_value {
            // It may happen that the balance is not as expected due to the
            // concurrent scenarios using this account.
            self.warn(&format!(
                "It doesn't seem like the transfer has reached receiver. \
                Receiver's balance before: {} and after: {}. Transfer value: {}",
                receiver_balance_before, receiver_balance_after, self.transfer_value,
            ));
        }

        self.info("Done");
        Ok(())
    }

    fn ident(&self) -> Ident {
        self.ident.clone()
    }
}
