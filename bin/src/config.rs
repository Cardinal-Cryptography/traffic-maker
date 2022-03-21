use clap::Parser;

#[derive(Debug, Parser, Clone)]
#[clap(version = "1.0")]
pub struct Config {
    /// WS endpoint address of the node to connect to
    #[clap(long, default_value = "127.0.0.1:9944")]
    pub node: String,

    /// Where to expose stats
    #[structopt(long, default_value = "127.0.0.1:8080")]
    pub expose_host: String,
}
