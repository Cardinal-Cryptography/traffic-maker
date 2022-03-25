# Traffic Maker 🚦

The code in this repository aims at filling two objectives:
1. real-world **traffic simulation** on a blockchain - obviously in a simplified way
2. **comprehensive testing** of all available features and functionalities - kinda E2E-tests on steroids

## Installation and usage
Apart from Rust version specified in `rust-toolchain` you do not need anything more.

To run the default scenario suite, you need a running chain with some node accessible via ws port.
Then you can run:
```shell
cargo build --release
./target/release/bin
```
The configuration file is `Timetable.toml`.
There you can specify:
 - `node` (by default `127.0.0.1:9944`): it is the web socket address to which bots will connect
 - `protocol` (by default `ws://`): you can specify whether to use ssl
 - `expose_host` (by default `127.0.0.1:8080`): where to publish statistics
 - which bots to launch

**Note:** To avoid accessing sudo, the code assumes that there is an account with seed `//Cornucopia` (public key: `5Cyo51qgfmVMva6R98KCiFQvPSjSEwWjKWLZnmBEDDC6eHkL`) with _a lot_ of balance.

Statistics are exposed at `expose_host` address, which by default is `127.0.0.1:8080`.
Data is served at two endpoints: `/details` (brief information about every launched scenario) and `/logs` (logs from particular scenario).

## Development

### Project structure
 - `bin` (binary) crate serves for just launching the application from command line and providing environment configuration
 - `chain-support` crate is a library providing functionality for chain interaction, useful in scenario development
 - `scenarios` is a collection of independent crates, each focusing on different testing aspect; something like `frame` directory for pallets crates in Substrate repository
 - `traffic` lib crate is responsible for scheduling and launching scenarios
 
### Adding new scenarios and deployment
TBA
