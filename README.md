# Traffic Maker

The code in this repository aims at filling two objectives:
  1. real-world **traffic simulation** on a blockchain - obviously, in a simplified way
  2. **comprehensive testing** of all available features and functionalities - kinda E2E-tests on steroids

## Installation and usage

Apart from Rust version specified in [`rust-toolchain`](rust-toolchain) you do not need anything more installed.

### Launching the default setting

To run the default scenario suite, you need a running chain with some node accessible via ws port.
By default, bots will try communicating with `127.0.0.1:9944`.
In order to change that, see the [Adjusting configuration](#adjusting-configuration) section.

For most actions, bots will need some tokens to pay fees.
To set up test accounts with funds run:

```shell
$ make setup
```

Afterwards, to start the test worker run:

```shell
$ make run
```

### Launching with default settings (docker)

If you prefer running bots in a docker container, you can call:

```shell
$ make setup
$ make docker
```
This will launch `docker-compose` accoring to [`docker/docker-compose.yml`](docker/docker-compose.yml).
If your chain is running on `localhost`, you will have to exchange line `node = "127.0.0.1:9944"` with `node = "host.docker.internal:9944"` in [`Timetable.toml`](Timetable.toml).

**Note:** To enable backend to communicate with local chain from docker you will need to run node with `--unsafe-rpc-external` and `--unsafe-ws-external` flags.

In addition, monitoring service will be served at `localhost:8080`.

### Adjusting configuration

#### Scenario configuration and scheduling

The main file is [`Timetable.toml`](Timetable.toml). There you can specify:

  - `node` (by default `127.0.0.1:9944`): it is the web socket address to which bots will connect
  - `expose_host` (by default `0.0.0.0:8080`): address where statistics are published
  - which bots to launch and their parameters

Statistics are exposed at two endpoints under the `expose_host` address.
Main data is served at `/details` (brief information about every launched scenario) and logs from particular scenarios are displayed at `/logs/<scenario identifier>`.

Each scenario configuration contains three obligatory fields:

  - `kind` - this specifies which scenario to run
  - `ident` - a unique identifier of this particular bot (you can launch multiple bots of the same `kind`, but they have to be distinguishable by `ident`)
  - `interval` - (human-readable) amount of time that should pass between finishing a run and the subsequent launch

Apart from that, most scenarios have some parameters (like strategy or scale) which you can tweak.

#### Account endowments

You can configure the initial endowments in [`set_up/Config.toml`](/set_up/Config.toml).
For details check [`set_up/README.md`](/set_up/README.md).

Accounts are given by their seeds.

_Note: In a production environment you usually do not want your bots to be exposed to access from anyone.
Therefore, before launching `make setup` and `make run` (`make run-docker`), you can set an environment variable `SECRET_PHRASE_SEED`.
Be aware that this is a **compile-time variable**, which means that changing it requires recompilation._

### Monitoring

You can use the provided GUI to browse the statistics by running:

```shell
$ make monitoring
```

This will start a [`trunk`](https://trunkrs.dev/) server with the GUI and open a new browser tab to take you there.
By default, it will be launched at `127.0.0.1:8040`.
To change its configuration, check [`monitoring/Trunk.toml`](monitoring/Trunk.toml).
For more details consult [`monitoring/README.md`](monitoring/README.md).

## Development

### Project structure

  - [`common`](common) (lib crate): contains the heart of this repo, i.e. the `Scenario` trait together with some useful utilities around it
  - [`chain-support`](chain-support) (lib crate): is a library providing functionality for chain interaction, useful in scenario development;
serves as a wrapper around [`aleph-client`](https://github.com/Cardinal-Cryptography/aleph-node/tree/main/aleph-client) and [`substrate-api-client`](https://github.com/scs/substrate-api-client)
  - [`traffic`](traffic) (lib crate): is responsible for scheduling and launching scenarios
  - [`scenarios`](scenarios) (collection of lib crates): gathers independent crates, each focusing on testing a different aspect; 
something like [`frame`](https://github.com/paritytech/substrate/tree/master/frame) directory for pallets crates in the Substrate repository
  - [`bin`](bin) (binary crate): serves for launching the application from command line and providing environment configuration;
additionally starts a web server exposing statistics
  - [`monitoring`](monitoring) (bin crate): (web) frontend part of application
  - [`set_up`](set_up) (bin crate): is responsible for endowing bot accounts
 
### Building docker image

Just run:
```shell
# for backend image (it will be tagged `traffic-maker`)
$ make build-backend-docker
# or for frontend image (it will be tagged `traffic-maker-monitoring`)
$ make build-frontend-docker
```

### Adding new scenarios

In order to implement a new scenario, follow these steps:

  1. If its scope fits into any existing subdirectories (crates) in [`scenarios`](scenarios), add a new module there (and register it in a corresponding `lib.rs`)
Otherwise, you will need to create a new crate.
It will need to be registered in the default workspace ([`Cargo.toml`](Cargo.toml)).
Also, you will have to add it as a dependency to [`bin/Cargo.toml`](bin/Cargo.toml)
  2. Write your scenario by implementing `Scenario` from the [`common`](common) crate.
Instantiating an object of your class should be done in an analogous way to other scenarios (through a mirror data structure, see e.g. [`scenarios/transfer/src/simple_transfer.rs`](scenarios/transfer/src/simple_transfer.rs)).
  3. Enable creating your scenario from a configuration file.
For this, extend `enum ScenarioConfig` in [`bin/src/config.rs`](bin/src/config.rs) and a corresponding method there (`construct_scenario`).
You should just follow the existing code and prepare very similar handling.
  4. If you need some accounts to have an initial balance, see the [Account endowments](#account-endowments) section.

### Deploying to Testnet

After your PR is merged into main, a new docker image (containing only backend part) will be built and pushed to a private registry (AWS ECR).
To launch it on a server (we have a dedicated EC2 instance for that), you will need to ssh there and run the new image manually.
For details contact @pmikolajczyk41 or @skrolikowski-10c.
