# Starknet-Foundry 🔨

Blazingly fast implementation of Foundry for developing Starknet contracts designed & developed by ex [Protostar](https://github.com/software-mansion/protostar) team from [Software Mansion](https://github.com/software-mansion/protostar)

## Installation

To install Starknet-Foundry, run:

```shell
$ curl -L https://raw.githubusercontent.com/foundry-rs/starknet-foundry/master/install.sh | sh -s -- starknet-foundry
```

If you want to specify a version, run the following command with the requested version:

```shell
$ curl -L https://raw.githubusercontent.com/foundry-rs/starknet-foundry/master/install.sh | sh -s -- starknet-foundry -v 0.1.0
```

To check if the Starknet-Foundry is installed correctly, run `forge --version` and `cast --version`.

## Development

### Environment setup

1. Install the latest [Rust](https://www.rust-lang.org/tools/install) version.
If you already have Rust installed make sure to upgrade it by running
```shell
$ rustup update
```
2. Clone this repository
3. Verify your setup by running [tests](#testing)
4. Build Starknet Foundry
```shell
$ cd ./starknet-foundry && cargo build --bins --release
```

### Testing
Test scripts require you to have asdf installed. 
Cast's tests require devenet as well.
Moreover, `./scripts/prepare-for-tests.sh` should be run once after setting up the development environment.

```bash
$ ./scripts/test_forge.sh
$ ./scripts/test_cast.sh
```
