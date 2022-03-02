use std::{thread::sleep, time::Duration};

use log::warn;
use sp_core::{crypto::AccountId32, sr25519::Pair};
use substrate_api_client::{rpc::WsRpcClient, Api};

pub mod account;
pub mod transfer;

pub type Connection = Api<Pair, WsRpcClient>;

/// Core struct representing an entity on the blockchain.
#[derive(Clone)]
pub struct Account {
    pub keypair: Pair,
    pub address: AccountId32,
}

pub fn create_connection(address: &str) -> Connection {
    let client = WsRpcClient::new(address);
    match Api::<Pair, _>::new(client) {
        Ok(api) => api,
        Err(why) => {
            warn!("Cannot create connection ({:?}), will try again in 1s", why);
            sleep(Duration::from_secs(1));
            create_connection(address)
        }
    }
}

#[macro_export]
macro_rules! send_extrinsic {
	($connection: expr,
	$module: expr,
	$call: expr
	$(, $args: expr) *) => {
		{
            use substrate_api_client::{compose_extrinsic, UncheckedExtrinsicV4, XtStatus};

            let tx: UncheckedExtrinsicV4<_> = compose_extrinsic!(
                $connection,
                $module,
                $call
                $(, ($args)) *
            );

            let _tx_hash = $connection
                .send_extrinsic(tx.hex_encode(), XtStatus::Finalized)
                .unwrap()
                .expect("Could not get tx hash");

            tx
		}
    };
}
