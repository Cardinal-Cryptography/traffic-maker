use chain_support::{parse_to_protocol, Protocol};
use clap::Parser;

#[derive(Debug, Parser, Clone)]
#[clap(version = "1.0")]
pub struct Config {
    /// WS endpoint address of the node to connect to
    #[clap(long, default_value = "127.0.0.1:9944")]
    pub node: String,

    /// Protocol to be used for connecting to node (`ws` or `wss`).
    #[structopt(name = "use_ssl", parse(from_flag = parse_to_protocol))]
    pub protocol: Protocol,

    /// Where to expose stats
    #[structopt(long, default_value = "127.0.0.1:8080")]
    pub expose_host: String,
}
