use sp_core::Pair;
use substrate_api_client::GenericAddress;

use crate::{send_extrinsic, Account, Connection};

/// Wrapper for the extrinsic `Balances::transfer`.
pub fn transfer(connection: &Connection, from: &Account, to: &Account, amount: u128) {
    let connection = connection.clone().set_signer(from.keypair.clone());

    send_extrinsic!(
        connection,
        "Balances",
        "transfer",
        GenericAddress::Id(to.address.clone()),
        Compact(amount)
    );
}
