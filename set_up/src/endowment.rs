use aleph_client::{
    balances_batch_transfer, keypair_from_string, send_xt, substrate_api_client, AnyConnection,
    RootConnection, SignedConnection,
};
use codec::Compact;
use serde::Deserialize;
use substrate_api_client::{
    compose_call, compose_extrinsic, AccountId, GenericAddress, Pair, XtStatus::Finalized,
};

use chain_support::{keypair_derived_from_seed, real_amount};

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

fn batch_set_endowment(connection: &RootConnection, accounts: Vec<AccountId>, amount: u128) {
    let metadata = connection.as_connection().metadata;
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
    let xt = compose_extrinsic!(connection.as_connection(), "Utility", "batch", xts);
    send_xt(connection, xt, Some("Set endowment"), Finalized);
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

pub fn perform_endowments(cli_config: &CliConfig, endowments: &[Endowment]) {
    let endowments = endowments
        .iter()
        .map(|Endowment { amount, accounts }| (real_amount(amount), flatten_accounts(accounts)));
    let performer = keypair_from_string(&*cli_config.phrase);

    if cli_config.transfer {
        let connection = SignedConnection::new(&cli_config.node, performer);
        for (amount, accounts) in endowments {
            balances_batch_transfer(&connection, accounts, amount);
        }
    } else {
        let connection = RootConnection::new(&cli_config.node, performer);
        for (amount, accounts) in endowments {
            batch_set_endowment(&connection, accounts, amount);
        }
    }
}
