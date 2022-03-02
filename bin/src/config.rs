use clap::Parser;

#[derive(Debug, Parser, Clone)]
#[clap(version = "1.0")]
pub struct Config {
    /// WS endpoint address of the node to connect to
    #[clap(long, default_value = "ws://127.0.0.1:9944")]
    pub node: String,
}
