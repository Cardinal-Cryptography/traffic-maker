use aleph_client::Connection;
use codec::Compact;
use serde::Deserialize;
use substrate_api_client::{
    compose_call, compose_extrinsic, AccountId, GenericAddress, Pair, XtStatus, XtStatus::Finalized,
};

use chain_support::{
    create_connection, keypair_derived_from_seed, keypair_from_string, real_amount, try_send_xt,
};

use crate::CliConfig;

#[derive(Clone, Debug, Deserialize)]
pub struct Account {
    pub name: String,
    pub copies: Option<usize>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Endowment {
    /// As `toml` does not support deserializing `u128`, so we need to operate
    /// on amounts scaled by `DECIMALS`.
    pub amount: u64,
    pub accounts: Vec<Account>,
}

fn batch_set_endowment(connection: &Connection, accounts: Vec<AccountId>, amount: u128) {
    let metadata = connection.metadata.clone();
    let xts = accounts
        .iter()
        .map(|account| {
            let endowment_call = compose_call!(
                metadata,
                "Balances",
                "set_balance",
                GenericAddress::Id(account.clone()),
                Compact(amount), // free balance
                Compact(0u128)   // reserved balance
            );
            compose_call!(metadata, "Sudo", "sudo", endowment_call)
        })
        .collect::<Vec<_>>();
    let xt = compose_extrinsic!(connection, "Utility", "batch", xts);
    try_send_xt(connection, xt, Some("Set endowment"), Finalized)
        .unwrap()
        .unwrap();
}

fn flatten_accounts(accounts: &[Account]) -> Vec<AccountId> {
    accounts
        .iter()
        .flat_map(|a| {
            if a.copies.is_none() {
                vec![a.name.clone()]
            } else {
                (0..a.copies.unwrap())
                    .map(|i| format!("{}{}", a.name, i))
                    .collect()
            }
        })
        .map(keypair_derived_from_seed)
        .map(|kp| kp.public())
        .map(AccountId::from)
        .collect()
}

fn batch_transfer(connection: &Connection, account_keys: Vec<AccountId>, endowment: u128) {
    let batch_endow = account_keys
        .into_iter()
        .map(|account_id| {
            compose_call!(
                connection.metadata,
                "Balances",
                "transfer",
                GenericAddress::Id(account_id),
                Compact(endowment)
            )
        })
        .collect::<Vec<_>>();

    let xt = compose_extrinsic!(connection, "Utility", "batch", batch_endow);
    try_send_xt(
        connection,
        xt,
        Some("batch of endow balances"),
        XtStatus::InBlock,
    )
    .unwrap();
}

pub fn perform_endowments(cli_config: &CliConfig, endowments: &[Endowment]) {
    let endowments = endowments
        .iter()
        .map(|Endowment { amount, accounts }| (real_amount(amount), flatten_accounts(accounts)));
    let performer = keypair_from_string(&*cli_config.phrase);

    let connection = create_connection(&cli_config.node).set_signer(performer);

    if cli_config.transfer {
        for (amount, accounts) in endowments {
            batch_transfer(&connection, accounts, amount);
        }
    } else {
        for (amount, accounts) in endowments {
            batch_set_endowment(&connection, accounts, amount);
        }
    }
}
