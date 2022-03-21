use aleph_client::{create_connection, from as parse_to_protocol, send_xt, Protocol};
use chain_support::account::{get_cornucopia, new_account_from_seed};
use clap::Parser;
use codec::Compact;
use substrate_api_client::{
    compose_call, compose_extrinsic, GenericAddress, Pair, XtStatus::Finalized,
};

#[derive(Debug, Parser, Clone)]
#[clap(version = "1.0")]
pub struct Config {
    /// WS endpoint address of the node to connect to.
    #[clap(long, default_value = "127.0.0.1:9944")]
    node: String,

    /// Protocol to be used for connecting to node (`ws` or `wss`).
    #[structopt(name = "use_ssl", parse(from_flag = parse_to_protocol))]
    protocol: Protocol,

    /// Seed phrase of the Sudo account.
    #[structopt(long, default_value = "//Alice")]
    sudo_phrase: String,

    /// Seed phrase of the Sudo account
    ///
    /// By default, 10^26.
    #[structopt(long, default_value = "100000000000000000000000000")]
    cornucopia_balance: u128,
}

fn set_up_cornucopia(config: &Config) {
    let sudo = new_account_from_seed(&*config.sudo_phrase);
    let sudo_connection = create_connection(&config.node, config.protocol).set_signer(sudo.keypair);
    let cornucopia = get_cornucopia();

    let xt = compose_call!(
        sudo_connection.metadata,
        "Balances",
        "set_balance",
        GenericAddress::Id(cornucopia.address),
        Compact(config.cornucopia_balance), // free balance
        Compact(0u128)                      // reserved balance
    );
    let xt = compose_extrinsic!(sudo_connection, "Sudo", "sudo", xt);

    send_xt(
        &sudo_connection,
        xt.hex_encode(),
        "Set up cornucopia",
        Finalized,
    );
}

fn main() {
    let config: Config = Config::parse();
    set_up_cornucopia(&config);
}
