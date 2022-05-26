use clap::Parser;

#[derive(Debug, Parser, Clone)]
#[clap(version = "1.0")]
pub struct CliConfig {
    /// WS endpoint address of the node to connect to.
    #[clap(long, default_value = "ws://127.0.0.1:9944")]
    pub node: String,

    /// Seed phrase of the account performing actions. If `transfer` is `false`, then it must be
    /// the sudo seed.
    #[clap(long, default_value = "//Alice")]
    pub phrase: String,

    /// Path to the config file.
    #[clap(long, default_value = "Config.toml")]
    pub config_file: String,

    /// If this flag is set, then initial balances are transferred from sudo account.
    /// Otherwise, they are set with `set_balance` extrinsic.
    #[clap(long)]
    pub transfer: bool,
}
