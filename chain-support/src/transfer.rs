use aleph_client::send_xt;
use sp_core::Pair;
use substrate_api_client::{compose_extrinsic, GenericAddress, XtStatus::Finalized};

use crate::{Account, Connection};

/// Wrapper for the extrinsic `Balances::transfer`.
pub fn transfer(connection: &Connection, from: &Account, to: &Account, amount: u128) {
    let connection = connection.clone().set_signer(from.keypair.clone());
    let xt = compose_extrinsic!(
        connection,
        "Balances",
        "transfer",
        GenericAddress::Id(to.address.clone()),
        Compact(amount)
    );
    send_xt(&connection, xt.hex_encode(), "transfer", Finalized);
}
