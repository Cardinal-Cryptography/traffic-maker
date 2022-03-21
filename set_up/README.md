## Setting up environment

This tool provides support for initializing local environment and adjusting chain for traffic-maker.


### Setting up Cornucopia

In order to avoid relying on sudo account for setting balances for bot-accounts, we need an account with 'unlimited' tokens.
We call it _cornucopia_.
Its seed phrase is `//Cornucopia`.

While deploying on Testnet, we will need a sudo help for initializing this one account.
However, for a local development and tests, we can do it ourselves, as usually, `//Alice` is the sudoer.

Instead of clicking in the web UI, you can just run this tool.

### Usage
```
set-up 1.0

USAGE:
    set-up [OPTIONS] [use_ssl]

ARGS:
    <use_ssl>    Protocol to be used for connecting to node (`ws` or `wss`)

OPTIONS:
        --cornucopia-balance <CORNUCOPIA_BALANCE>
            Seed phrase of the Sudo account [default: 100000000000000000000000000]

    -h, --help
            Print help information

        --node <NODE>
            WS endpoint address of the node to connect to [default: 127.0.0.1:9944]

        --sudo-phrase <SUDO_PHRASE>
            Seed phrase of the Sudo account [default: //Alice]

    -V, --version
            Print version information
```
So by default, if your local chain exposes its web socket at `ws://127.0.0.1:9944` and `//Alice` is the sudo phrase, then running just:
```
cargo run
```
will set up `//Cornucopia` account with 10<sup>26</sup> balance.
