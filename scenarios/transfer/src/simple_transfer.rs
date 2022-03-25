use std::time::Duration;

use log::info;

use chain_support::{
    account::{get_free_balance, new_account_from_seed, top_up},
    transfer::transfer,
    Account, Connection,
};
use common::{Ident, Scenario};

/// Keeps two accounts: `sender` and `receiver`. Once in the `interval`,
/// `sender` sends `transfer_value` units to `receiver`.
#[derive(Clone)]
pub struct SimpleTransferScenario {
    ident: Ident,
    sender: Account,
    receiver: Account,
    interval: Duration,
    transfer_value: u128,
    connection: Connection,
}

impl SimpleTransferScenario {
    pub fn new(connection: &Connection, ident: Ident, interval: Duration) -> Self {
        let sender = new_account_from_seed("//SimpleTransferSender");
        let receiver = new_account_from_seed("//SimpleTransferReceiver");

        let transfer_value = 1_000_000_000;

        top_up(connection, &sender, transfer_value * 1_000);

        SimpleTransferScenario {
            ident,
            sender,
            receiver,
            interval,
            transfer_value,
            connection: connection.clone(),
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
        let receiver_balance_before = get_free_balance(&self.receiver, &self.connection);
        transfer(
            &self.connection,
            &self.sender,
            &self.receiver,
            self.transfer_value,
        );
        let receiver_balance_after = get_free_balance(&self.receiver, &self.connection);
        info!(target: self.ident.0.as_str(), "Almost done");

        receiver_balance_after == receiver_balance_before + self.transfer_value
    }

    fn ident(&self) -> Ident {
        self.ident.clone()
    }
}
