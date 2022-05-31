# Traffic-maker: Setting up the environment

This tool provides support for initializing local environment and adjusting chain for traffic-maker.

## Endowing accounts

Accounts that are used in scenarios are supposed to already have sufficient balances.
Through the [`Config.toml`](Config.toml) file we can specify a starting balance for any needed account.
While deploying on Testnet, most probably you will need to have a sudo privileges help for doing this.
However, for local development and tests, we can do it ourselves, as usual, `//Alice` is the sudoer, and we have full freedom.

### Usage

```shell
set-up 1.0

USAGE:
    set-up [OPTIONS]

OPTIONS:
        --config-file <CONFIG_FILE>    Path to the config file [default: Config.toml]
    -h, --help                         Print help information
        --node <NODE>                  WS endpoint address of the node to connect to [default:
                                       ws://127.0.0.1:9944]
        --phrase <PHRASE>              Seed phrase of the account performing actions. If `transfer`
                                       is `false`, then it must be the sudo seed [default: //Alice]
        --transfer                     If this flag is set, then initial balances are transferred
                                       from sudo account. Otherwise, they are set with `set_balance`
                                       extrinsic
    -V, --version                      Print version information
```
So by default, if your local chain exposes its web socket at `ws://127.0.0.1:9944` and `//Alice` is the sudo phrase, then running just:
```
cargo run
```
will endow all accounts specified in [`Config.toml`](Config.toml) with a corresponding amount.
The endowment will be performed with `set_balance` extrinsic.

If you want to avoid changing total issuance on the chain, you can pass `--transfer` flag:
```
cargo run -- --transfer
```
Then, the account generated from `phrase` will transfer its balance instead of minting new tokens.

### Configuration

In [`Config.toml`](Config.toml) you can add a new section called `[[endowments]]` where you specify the amount and derivation paths for target accounts.
Note, that the amount is interpreted as full tokens, i.e. this number will be further scaled up by 10<sup>12</sup>.

All accounts are obtained by appending specified derivation path to a special secret phrase.
By default, it is an empty string, but you can provide your own by setting environment variable `SECRET_PHRASE_SEED`.

**Note** `SECRET_PHRASE_SEED` is a **compile-time** variable.
This means that the compiled binary will use the seed that was set during compilation time.
In particular running `cargo run` after changing the seed will trigger recompilation.
