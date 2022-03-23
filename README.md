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
./target/release/bin --node "<address of the node>" --expose_host "<address for stats>"
```
The default node address is `127.0.0.1:9944`, so for that you do not need to specify `--node` option.

**Note:** For now, only `ws` ports are achievable. For `wss` support we must wait for changes to `aleph-client`.

Currently, there is only one scenario available - `SimpleTransferScenario`, running every 15 seconds.
More scenarios and easy configuration are coming soon.

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
