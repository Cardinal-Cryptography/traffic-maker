use std::time::Duration;

use aleph_client::{balances_transfer, get_free_balance, Connection};
use substrate_api_client::{AccountId, Pair, XtStatus};

use chain_support::keypair_derived_from_seed;
use common::{Ident, Scenario, ScenarioError, ScenarioLogging};

const TRANSFER_VALUE: u128 = 1_000_000_000;

/// Keeps two accounts: `sender` and `receiver`. Once in the `interval`,
/// `sender` sends `transfer_value` units to `receiver`.
#[derive(Clone)]
pub struct SimpleTransferScenario {
    ident: Ident,
    receiver: AccountId,
    interval: Duration,
    connection: Connection,
}

impl SimpleTransferScenario {
    pub fn new(connection: &Connection, ident: Ident, interval: Duration) -> Self {
        let sender = keypair_derived_from_seed("//SimpleTransferSender");
        let connection = connection.clone().set_signer(sender);

        let receiver =
            AccountId::from(keypair_derived_from_seed("//SimpleTransferReceiver").public());

        SimpleTransferScenario {
            ident,
            receiver,
            interval,
            connection,
        }
    }
}

#[async_trait::async_trait]
impl Scenario for SimpleTransferScenario {
    fn interval(&self) -> Duration {
        self.interval
    }

    async fn play(&mut self) -> Result<(), ScenarioError> {
        self.info("Ready to go");

        let receiver_balance_before = get_free_balance(&self.connection, &self.receiver);
        balances_transfer(
            &self.connection,
            &self.receiver,
            TRANSFER_VALUE,
            XtStatus::Finalized,
        );
        let receiver_balance_after = get_free_balance(&self.connection, &self.receiver);

        if receiver_balance_after != receiver_balance_before + TRANSFER_VALUE {
            // It may happen that the balance is not as expected due to the
            // concurrent scenarios using this account.
            self.warn(&format!(
                "It doesn't seem like the transfer has reached receiver. \
                Receiver's balance before: {} and after: {}. Transfer value: {}",
                receiver_balance_before, receiver_balance_after, TRANSFER_VALUE,
            ));
        }

        self.info("Done");
        Ok(())
    }

    fn ident(&self) -> Ident {
        self.ident.clone()
    }
}
