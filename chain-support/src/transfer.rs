use std::{thread::sleep, time::Duration};

use aleph_client::{try_send_xt, Connection};
use substrate_api_client::{AccountId, GenericAddress, XtStatus};

use common::ScenarioError;

pub fn try_transfer(
    connection: &Connection,
    target: &AccountId,
    amount: u128,
) -> Result<(), ScenarioError> {
    for _ in 0..5 {
        let xt = connection.balance_transfer(GenericAddress::Id(target.clone()), amount);
        if try_send_xt(connection, xt, Some("transfer"), XtStatus::Finalized).is_ok() {
            return Ok(());
        }

        sleep(Duration::from_millis(500));
    }
    Err(ScenarioError::CannotSendExtrinsic)
}
