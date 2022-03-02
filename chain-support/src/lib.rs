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
