// Needed for `do_async!`.
#![feature(fn_traits)]

use std::{env, fmt::Display, thread::sleep, time::Duration};

pub use aleph_client::{send_xt, Connection, KeyPair};
use codec::Encode;
use log::{info, warn};
use sp_core::H256;
pub use substrate_api_client;
use substrate_api_client::{
    error::Error, rpc::WsRpcClient, AccountId, ApiResult, Pair, UncheckedExtrinsicV4, XtStatus,
};

pub use event_listening::{
    with_event_listening, with_event_matching, Event, EventKind, ListeningError,
    SingleEventListener, Transfer as TransferEvent,
};

mod event_listening;
mod macros;

pub fn keypair_from_string(seed: &str) -> KeyPair {
    KeyPair::from_string(seed, None).expect("Can't create pair from seed value")
}

pub fn account_from_keypair(keypair: &KeyPair) -> AccountId {
    AccountId::from(keypair.public())
}

pub fn create_connection(address: &str) -> Connection {
    let client = WsRpcClient::new(address);
    match Connection::new(client) {
        Ok(api) => api,
        Err(why) => {
            warn!(
                "[+] Can't create_connection because {:?}, will try again in 1s",
                why
            );
            sleep(Duration::from_millis(1000));
            create_connection(address)
        }
    }
}

pub fn try_send_xt<T: Encode + sp_core::Encode>(
    connection: &Connection,
    xt: UncheckedExtrinsicV4<T>,
    xt_name: Option<&'static str>,
    xt_status: XtStatus,
) -> ApiResult<Option<H256>> {
    let hash = connection
        .send_extrinsic(xt.hex_encode(), xt_status)?
        .ok_or_else(|| Error::Other(String::from("Could not get tx/block hash").into()))?;

    match xt_status {
        XtStatus::Finalized | XtStatus::InBlock => {
            info!(target: "aleph-client",
                "Transaction `{}` was included in block with hash {}.",
                xt_name.unwrap_or_default(), hash);
            Ok(Some(hash))
        }
        _ => Ok(None),
    }
}

/// Creates a new derived `KeyPair` from provided `seed` as a derivation path.
///
/// The base seed is empty by default, but can be overridden with env `SECRET_PHRASE_SEED`.
/// Assumes that `seed` is already prefixed with a derivation delimiter (either `/` or `//`).
pub fn keypair_derived_from_seed<S: AsRef<str> + Display>(seed: S) -> KeyPair {
    let base_seed = env::var("SECRET_PHRASE_SEED").unwrap_or_default();
    let full_seed = format!("{}{}", base_seed, seed);
    keypair_from_string(&*full_seed)
}

/// A single token is 10^12 rappens. This value corresponds to the constants defined in
/// `aleph-node::primitives` (`TOKEN_DECIMALS` and `TOKEN`).
pub const DECIMALS: u128 = 1_000_000_000_000;

pub const fn real_amount(scaled: &u64) -> u128 {
    *scaled as u128 * DECIMALS
}
