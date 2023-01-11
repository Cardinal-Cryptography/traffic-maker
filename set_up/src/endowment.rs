use aleph_client::{
    account_from_keypair,
    aleph_runtime::RuntimeCall::{Balances, Sudo},
    keypair_from_string,
    pallet_balances::pallet::Call::set_balance,
    pallet_sudo::pallet::Call::sudo_unchecked_weight,
    pallets::{balances::BalanceUserBatchExtApi, utility::UtilityApi},
    sp_weights::weight_v2::Weight,
    AccountId, RootConnection, SignedConnection, TxStatus,
};
use chain_support::{keypair_derived_from_seed, real_amount};
use serde::Deserialize;

use crate::CliConfig;

#[derive(Clone, Debug, Deserialize)]
pub struct AccountSeries {
    pub name: String,
    pub copies: Option<usize>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Endowment {
    /// As `toml` does not support deserializing `u128`, so we need to operate
    /// on amounts scaled by `DECIMALS`.
    pub amount: u64,
    pub accounts: Vec<AccountSeries>,
}

async fn batch_set_endowment(connection: &RootConnection, accounts: Vec<AccountId>, amount: u128) {
    let subcalls = accounts
        .iter()
        .map(|account| {
            Sudo(sudo_unchecked_weight {
                call: Box::new(Balances(set_balance {
                    who: account.clone().into(),
                    new_free: amount,
                    new_reserved: 0,
                })),
                weight: Weight {
                    ref_time: 0,
                    proof_size: 0,
                },
            })
        })
        .collect();

    connection
        .batch_call(subcalls, TxStatus::Finalized)
        .await
        .expect("Failed to endow accounts");
}

fn flatten_accounts(accounts: &[AccountSeries]) -> Vec<AccountId> {
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
        .map(|kp| account_from_keypair(kp.signer()))
        .collect()
}

pub async fn perform_endowments(cli_config: &CliConfig, endowments: &[Endowment]) {
    let endowments = endowments
        .iter()
        .map(|Endowment { amount, accounts }| (real_amount(amount), flatten_accounts(accounts)));
    let performer = keypair_from_string(&cli_config.phrase);

    if cli_config.transfer {
        let connection = SignedConnection::new(&cli_config.node, performer).await;
        for (amount, accounts) in endowments {
            connection
                .batch_transfer(&accounts, amount, TxStatus::Finalized)
                .await
                .expect("Failed to endow accounts");
        }
    } else {
        let connection = RootConnection::new(&cli_config.node, performer)
            .await
            .expect("Failed to create root connection. Check your phrase.");
        for (amount, accounts) in endowments {
            batch_set_endowment(&connection, accounts, amount).await;
        }
    }
}
