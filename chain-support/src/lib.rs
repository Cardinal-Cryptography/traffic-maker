// Needed for `do_async!`.
#![feature(fn_traits)]

use std::{env, fmt::Display};

pub use aleph_client::{
    create_connection, keypair_from_string, send_xt, try_send_xt, AnyConnection, Connection,
    KeyPair, RootConnection, SignedConnection,
};

pub use event_listening::{
    with_event_listening, with_event_matching, Event, EventKind, ListeningError,
    SingleEventListener, Transfer as TransferEvent,
};

mod event_listening;
mod macros;

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
