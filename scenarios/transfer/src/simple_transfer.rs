use std::time::Duration;

use chain_support::{
    account::{get_free_balance, new_account_from_seed, top_up},
    transfer::transfer,
    Account, Connection,
};
use traffic::Scenario;

/// Keeps two accounts: `sender` and `receiver`. Once in the `interval`,
/// `sender` sends `1` unit to `receiver`.
#[derive(Clone)]
pub struct SimpleTransferScenario {
    sender: Account,
    receiver: Account,
    interval: Duration,
    connection: Connection,
}

impl SimpleTransferScenario {
    pub fn new(connection: &Connection, interval: Duration) -> Self {
        let sender = new_account_from_seed("//SimpleTransferSender");
        let receiver = new_account_from_seed("//SimpleTransferReceiver");

        top_up(connection, &sender, 100_000_000_000);

        SimpleTransferScenario {
            sender,
            receiver,
            interval,
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
        let receiver_balance_before = get_free_balance(&self.receiver, &self.connection);
        transfer(&self.connection, &self.sender, &self.receiver, 1);
        let receiver_balance_after = get_free_balance(&self.receiver, &self.connection);

        receiver_balance_after == receiver_balance_before + 1
    }

    fn ident(&self) -> &str {
        "SimpleTransfer"
    }

    fn immediate(&self) -> bool {
        true
    }
}
