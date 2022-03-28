use aleph_client::{from as parse_to_protocol, Protocol};
use clap::Parser;

#[derive(Debug, Parser, Clone)]
#[clap(version = "1.0")]
pub struct CliConfig {
    /// WS endpoint address of the node to connect to.
    #[clap(long, default_value = "127.0.0.1:9944")]
    pub node: String,

    /// Protocol to be used for connecting to node (`ws` or `wss`).
    #[clap(name = "use_ssl", parse(from_flag = parse_to_protocol))]
    pub protocol: Protocol,

    /// Seed phrase of the Sudo account.
    #[clap(long, default_value = "//Alice")]
    pub sudo_phrase: String,

    /// Path to the config file.
    #[clap(long, default_value = "Config.toml")]
    pub config_file: String,

    /// If this flag is set, then initial balances are transferred from sudo account.
    /// Otherwise, they are set with `set_balance` extrinsic.
    #[clap(long)]
    pub transfer: bool,
}
