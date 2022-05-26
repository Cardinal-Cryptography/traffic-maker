[![rustfmt &amp; clippy](https://github.com/Cardinal-Cryptography/traffic-maker/actions/workflows/clippy-and-fmt.yml/badge.svg)](https://github.com/Cardinal-Cryptography/traffic-maker/actions/workflows/clippy-and-fmt.yml)

# Traffic Maker

The code in this repository aims at filling two objectives:
1. real-world **traffic simulation** on a blockchain - obviously, in a simplified way
2. **comprehensive testing** of all available features and functionalities - kinda E2E-tests on steroids

## Installation and usage

Apart from Rust version specified in `rust-toolchain` you do not need anything more installed.

#### Launching the default setting

To run the default scenario suite, you need a running chain with some node accessible via ws port.

For most actions, bots will need some tokens to pay fees.
To set up test accounts with funds run:

```shell
$ make setup
```

Afterwards, to start the test worker run:

```shell
$ make run
```

#### Launching the default setting (docker)

If you prefer running bots in a docker container, you can call:

```shell
$ make 
```


Two configuration files control this. The first one is `set_up/Config.toml`, where you can configure the initial
endowments. The second one is `Timetable.toml`. There you can specify:

 - `node` (by default `127.0.0.1:9944`): it is the web socket address to which bots will connect
 - `protocol` (by default `ws://`): you can specify whether to use ssl
 - `expose_host` (by default `127.0.0.1:8080`): where to publish statistics
 - which bots to launch

Statistics are exposed at `expose_host` address, which by default is `127.0.0.1:8080`.
Data is served at two endpoints: `/details` (brief information about every launched scenario) and `/logs` (logs from particular scenario).

You can use the provided GUI to browse the statistics by running:

```shell
$ make monitoring
```

This will start a `trunk` server with the GUI and open a new browser tab to take you there.

## Development

### Project structure

 - `bin` (binary) crate serves for just launching the application from command line and providing environment configuration
 - `chain-support` crate is a library providing functionality for chain interaction, useful in scenario development
 - `scenarios` is a collection of independent crates, each focusing on different testing aspect; something like `frame` directory for pallets crates in Substrate repository
 - `traffic` lib crate is responsible for scheduling and launching scenarios
 
### Adding new scenarios and deployment

TBA
