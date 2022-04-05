use aleph_client::{
    balances_batch_transfer, create_connection, keypair_from_string, send_xt, Connection,
};
use codec::Compact;
use serde::Deserialize;
use substrate_api_client::{
    compose_call, compose_extrinsic, AccountId, GenericAddress, Pair, XtStatus::Finalized,
};

use chain_support::keypair_derived_from_seed;

use crate::{lib::real_amount, CliConfig};

#[derive(Clone, Debug, Deserialize)]
pub struct Account {
    pub name: String,
    pub copies: Option<usize>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Endowment {
    pub amount: u64,
    pub accounts: Vec<Account>,
}

fn set_endowment(sudo_connection: &Connection, account: AccountId, amount: u128) {
    // Unfortunately, `Connection::balance_set_balance` does not work. Calling it without wrapping
    // with `Sudo::sudo` results in `BadOrigin`, so the inner extrinsic must be created with
    // `compose_call` instead of `compose_extrinsic` as it is done in `Connection::balance_set_balance`.
    let xt = compose_call!(
        sudo_connection.metadata,
        "Balances",
        "set_balance",
        GenericAddress::Id(account),
        Compact(amount), // free balance
        Compact(0u128)   // reserved balance
    );
    let xt = compose_extrinsic!(sudo_connection, "Sudo", "sudo", xt);
    send_xt(sudo_connection, xt, Some("Set endowment"), Finalized);
}

pub fn perform_endowments(cli_config: &CliConfig, endowments: &[Endowment]) {
    let sudo = keypair_from_string(&*cli_config.sudo_phrase);
    let sudo_connection = create_connection(&cli_config.node).set_signer(sudo);

    for Endowment { amount, accounts } in endowments {
        let accounts = accounts
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
            .collect();

        if cli_config.transfer {
            balances_batch_transfer(&sudo_connection, accounts, real_amount(amount));
        } else {
            for account in accounts {
                set_endowment(&sudo_connection, account, real_amount(amount))
            }
        }
    }
}
