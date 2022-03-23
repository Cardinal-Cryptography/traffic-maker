use sp_core::{crypto::AccountId32, sr25519, Pair};
use substrate_api_client::Balance;

use crate::{Account, Connection};

impl Account {
    fn new(keypair: sr25519::Pair, address: AccountId32) -> Self {
        Account { keypair, address }
    }
}

/// Creates a new `Account` from provided seed.
pub fn new_account_from_seed(seed: &str) -> Account {
    let keypair: sr25519::Pair =
        Pair::from_string(seed, None).expect("Should create pair from seed value");

    Account::new(keypair.clone(), AccountId32::from(keypair.public()))
}

/// Creates a new derived `Account` from provided `seed` as a derivation path.
/// The base seed is empty by default, but can be overridden with env.
pub fn new_derived_account_from_seed(seed: &str) -> Account {
    let base_seed = option_env!("SECRET_PHRASE_SEED").unwrap_or("").to_string();
    let full_seed = format!("{}{}", base_seed, seed);
    new_account_from_seed(&*full_seed)
}

/// Returns free balance of `account`.
pub fn get_free_balance(account: &Account, connection: &Connection) -> Balance {
    match connection
        .get_account_data(&account.address)
        .expect("Should be able to access account data")
    {
        Some(account_data) => account_data.free,
        // Account may have not been initialized yet or liquidated due to the lack of funds.
        None => 0,
    }
}
