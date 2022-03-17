use sp_core::{crypto::AccountId32, sr25519, Pair};
use substrate_api_client::Balance;

use crate::{transfer::transfer, Account, Connection};

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

/// Returns the account that is supposed to have 'unlimited' balances. Kinda faucet.
///
/// Thanks to this we can set balances at will without using `sudo` account.
fn get_cornucopia() -> Account {
    new_account_from_seed("//Cornucopia")
}

/// Workaround for `set_balance` extrinsic. `amount` tokens are transferred
/// from //Cornucopia to `account`.
pub fn top_up(connection: &Connection, account: &Account, amount: u128) {
    let cornucopia = get_cornucopia();
    transfer(connection, &cornucopia, account, amount);
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
