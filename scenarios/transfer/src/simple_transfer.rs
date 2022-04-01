use std::time::Duration;

use aleph_client::{balances_transfer, get_free_balance, Connection};
use log::info;
use substrate_api_client::{AccountId, Pair, XtStatus};

use chain_support::keypair_derived_from_seed;
use common::{Ident, Scenario};

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

    async fn play(&mut self) -> bool {
        info!(target: self.ident.0.as_str(), "Ready to go");
        let receiver_balance_before = get_free_balance(&self.connection, &self.receiver);
        balances_transfer(
            &self.connection,
            &self.receiver,
            TRANSFER_VALUE,
            XtStatus::Finalized,
        );
        let receiver_balance_after = get_free_balance(&self.connection, &self.receiver);
        info!(target: self.ident.0.as_str(), "Almost done");

        receiver_balance_after == receiver_balance_before + TRANSFER_VALUE
    }

    fn ident(&self) -> Ident {
        self.ident.clone()
    }
}
