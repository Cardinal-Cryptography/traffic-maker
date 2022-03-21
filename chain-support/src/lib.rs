pub use aleph_client::{
    create_connection, from as parse_to_protocol, send_xt, Connection, Protocol,
};
use sp_core::{crypto::AccountId32, sr25519::Pair};

pub mod account;
pub mod transfer;

/// Core struct representing an entity on the blockchain.
#[derive(Clone)]
pub struct Account {
    pub keypair: Pair,
    pub address: AccountId32,
}
