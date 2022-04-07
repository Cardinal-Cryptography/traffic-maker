use std::time::Duration;

use aleph_client::{get_free_balance, try_send_xt, Connection};
use serde::Deserialize;
use substrate_api_client::{AccountId, GenericAddress, Pair, XtStatus::Finalized};

use chain_support::{do_async, keypair_derived_from_seed, real_amount};
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
            transfer_value: real_amount(&props.transfer_value),
            receiver,
            connection,
        }
    }
}

// It is quite hard to make macros work with associated methods.
fn transfer(
    connection: &Connection,
    target: &AccountId,
    transfer_value: u128,
) -> Result<(), ScenarioError> {
    for _ in 0..5 {
        let xt = connection.balance_transfer(GenericAddress::Id(target.clone()), transfer_value);
        if try_send_xt(connection, xt, Some("transfer"), Finalized).is_ok() {
            return Ok(());
        }
    }
    Err(ScenarioError::CannotSendExtrinsic)
}

#[async_trait::async_trait]
impl Scenario for SimpleTransfer {
    fn interval(&self) -> Duration {
        self.interval
    }

    async fn play(&mut self) -> Result<(), ScenarioError> {
        self.info("Ready to go");

        let receiver_balance_before: u128 =
            do_async!(get_free_balance, self.connection by ref, self.receiver by ref)?;

        match do_async!(transfer, self.connection by ref, self.receiver by ref, self.transfer_value)?
        {
            e @ Err(ScenarioError::CannotSendExtrinsic) => {
                self.error("Could not send extrinsic with transfer");
                return e;
            }
            e @ Err(_) => return e,
            _ => {}
        };

        let receiver_balance_after: u128 =
            do_async!(get_free_balance, self.connection by ref, self.receiver by ref)?;

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
