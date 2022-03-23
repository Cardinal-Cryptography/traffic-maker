use aleph_client::{create_connection, send_xt, Connection};
use codec::Compact;
use serde::Deserialize;
use substrate_api_client::{
    compose_call, compose_extrinsic, GenericAddress, Pair, XtStatus::Finalized,
};

use chain_support::{account::new_account_from_seed, transfer::transfer, Account};

use crate::{
    lib::{full_phrase, real_amount},
    CliConfig,
};

#[derive(Clone, Debug, Deserialize)]
pub struct Endowment {
    pub amount: u64,
    pub accounts: Vec<String>,
}

fn set_endowment(account: &str, amount: u64, sudo_connection: &Connection) {
    let beneficiary = new_account_from_seed(&*full_phrase(account));
    let xt = compose_call!(
        sudo_connection.metadata,
        "Balances",
        "set_balance",
        GenericAddress::Id(beneficiary.address),
        Compact(real_amount(amount)), // free balance
        Compact(0u128)                // reserved balance
    );
    let xt = compose_extrinsic!(sudo_connection, "Sudo", "sudo", xt);

    send_xt(sudo_connection, xt.hex_encode(), "Set endowment", Finalized);
}

fn transfer_endowment(account: &str, amount: u64, sudo_connection: &Connection, sudo: &Account) {
    let beneficiary = new_account_from_seed(&*full_phrase(&account.to_string()));
    transfer(sudo_connection, sudo, &beneficiary, real_amount(amount));
}

pub fn perform_endowments(cli_config: &CliConfig, endowments: &[Endowment]) {
    let sudo = new_account_from_seed(&*cli_config.sudo_phrase);
    let sudo_connection =
        create_connection(&cli_config.node, cli_config.protocol).set_signer(sudo.keypair.clone());

    for endowment in endowments.iter() {
        for account in endowment.accounts.iter() {
            if cli_config.transfer {
                transfer_endowment(account, endowment.amount, &sudo_connection, &sudo)
            } else {
                set_endowment(account, endowment.amount, &sudo_connection)
            }
        }
    }
}
