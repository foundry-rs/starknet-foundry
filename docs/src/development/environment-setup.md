# Environment Setup
> 💡 **Info**
> This tutorial is only relevant if you wish to contribute to Starknet Foundry. 
> If you plan to only use it as a tool for your project, you can skip this part.

## Prerequisites

### Rust

Install the latest stable [Rust](https://www.rust-lang.org/tools/install) version.
If you already have Rust installed make sure to upgrade it by running

```shell
$ rustup update
```

### Scarb
You can read more about installing Scarb [here](https://docs.swmansion.com/scarb/download.html).

Please make sure you're using Scarb installed via [asdf](https://asdf-vm.com/) - otherwise some tests may fail.
> To verify, run:
> 
> ```shell
> $ which scarb
> ```
> the result of which should be:
> ```shell
> $HOME/.asdf/shims/scarb
> ```
> 
> If you previously installed scarb using an official installer, you may need to remove this installation or modify your PATH to make sure asdf installed one is always used.

### cairo-profiler
You can read more
about installing `cairo-profiler` [here](https://github.com/software-mansion/cairo-profiler?tab=readme-ov-file#installation).

> ❗️ **Warning**
> 
> If you haven't pushed your branch to the remote yet (you've been working only locally) some tests may fail, including:
> 
> - `e2e::running::simple_package_with_git_dependency`
> 
> After pushing the branch to the remote, those tests should pass.

### Starknet Devnet
Install [starknet-devnet](https://github.com/0xSpaceShard/starknet-devnet) via [asdf](https://asdf-vm.com/).

### Universal sierra compiler
Install the latest [universal-sierra-compiler](https://github.com/software-mansion/universal-sierra-compiler) version.

## Running Tests
Tests can be run with:

```shell
$ cargo test
```

`cargo test` is the primary way to run tests locally.

> ❗️ **Warning**
>
> `cargo nextest run` may leave an orphan `starknet-devnet` process in some scenarios.
> For local runs, prefer `cargo test`.
> If you want to use `nextest`, find and `kill` orphan devnet process:
>
> ```shell
> $ ps aux | grep starknet-devnet | grep -v grep
> ```

## Cairo Native

To develop Starknet Foundry with Cairo Native support, you need to enable the `cairo-native` feature in Cargo and
install the required dependencies.

### LLVM

LLVM 19 is required to build forge with Cairo Native support and to run it.

#### macOS

On macOS in can be installed with

```shell
$ brew install llvm@19
```

Next, export the following environment variables:

```
LIBRARY_PATH=/opt/homebrew/lib
MLIR_SYS_190_PREFIX="$(brew --prefix llvm@19)"
LLVM_SYS_191_PREFIX="$(brew --prefix llvm@19)"
TABLEGEN_190_PREFIX="$(brew --prefix llvm@19)"
```

#### Linux

LLVM installation varies between distributions.
See [here](https://llvm.org/docs/GettingStarted.html) and [here](https://releases.llvm.org/download.html) for more
details.

Next, export the following environment variables, note that the paths may differe depending on your distribution and
installation method:

```
MLIR_SYS_190_PREFIX=/usr/lib/llvm-19
LLVM_SYS_191_PREFIX=/usr/lib/llvm-19
TABLEGEN_190_PREFIX=/usr/lib/llvm-19
```

## Ledger Testing

Ledger tests use [Speculos](https://github.com/LedgerHQ/speculos), a Ledger device emulator running inside Docker. No physical Ledger device is needed.

### Prerequisites

- [Docker](https://docs.docker.com/get-docker/) installed and running

### Setup

**1. Get the Ledger app binary**

The test suite expects a pre-built Nano X ELF binary at:

```
crates/sncast/tests/data/ledger-app/nanox.elf
```

Build it from the [app-starknet repository](https://github.com/LedgerHQ/app-starknet):

```shell
$ git clone --depth 1 --branch nanox_2.7.0_2.4.0_sdk_v26.0.2 https://github.com/LedgerHQ/app-starknet
$ docker run --rm \
    -v "$(pwd)/app-starknet:/app-starknet" \
    ghcr.io/ledgerhq/ledger-app-builder/ledger-app-dev-tools:5.3.14 \
    bash -c "cd /app-starknet/starknet && cargo ledger build nanox"
$ mkdir -p crates/sncast/tests/data/ledger-app
$ cp app-starknet/starknet/target/nanox/release/starknet \
    crates/sncast/tests/data/ledger-app/nanox.elf
```

### Running Ledger Tests

Tests run inside the `ledger-app-dev-tools` Docker image, which provides Speculos and all required tooling.

```shell
$ docker run --rm -it \
    -v "$(pwd):/workspace" \
    -v ledger_build_cache:/workspace/target \
    -v "$HOME/.cargo/registry:/root/.cargo/registry" \
    -v ledger_asdf_cache:/root/.asdf \
    -v ledger_local_cache:/root/.local \
    -w /workspace \
    -e CARGO_TARGET_DIR=/workspace/target \
    ghcr.io/ledgerhq/ledger-app-builder/ledger-app-dev-tools:5.3.14 \
    bash -c '
        [ -f /opt/.cargo/env ] && source /opt/.cargo/env
        if ! [ -x "$HOME/.asdf/shims/scarb" ]; then
            curl --proto "=https" --tlsv1.2 -sSf https://sh.starkup.sh | sh -s -- --yes
        fi
        export PATH="$HOME/.asdf/shims:$HOME/.asdf/bin:$HOME/.local/bin:$PATH"
        [ -f "$HOME/.asdf/asdf.sh" ] && source "$HOME/.asdf/asdf.sh"
        SCARB_VERSION=$(grep "scarb " /workspace/.tool-versions | cut -d " " -f 2)
        DEVNET_VERSION=$(grep "starknet-devnet " /workspace/.tool-versions | cut -d " " -f 2)
        asdf set -u scarb "$SCARB_VERSION"
        asdf set -u starknet-devnet "$DEVNET_VERSION"
        cargo test -p sncast --features ledger-emulator --test main ledger -- --nocapture --ignored
    '
```

> ❗️ **Note**
>
> The first run compiles the workspace and installs the Scarb toolchain via starkup. Expect it to take a while. Later runs reuse the Docker cache volumes and usually finish in a few minutes.
>
> Pasting the multi-line `docker run` directly into the terminal may break due to quoting or bracketed-paste behavior. If you hit problems, save the command to a file (e.g. `run_ledger_tests.sh`).

> 💡 **Tip: Clearing the cache**
>
> To wipe all cached volumes and start fresh:
>
> ```shell
> $ docker volume rm ledger_build_cache ledger_asdf_cache ledger_local_cache
> ```

> 💡 **Tip: Linux users**
>
> On Linux, [Speculos](https://speculos.ledger.com/installation/build.html) can be installed natively. In that case, tests can be run directly with:
>
> ```shell
> $ cargo test -p sncast --features ledger-emulator --test main ledger -- --nocapture --ignored
> ```

## Formatting and Lints

Starknet Foundry uses [rustfmt](https://github.com/rust-lang/rustfmt) for formatting. You can run the formatter with

```shell
$ cargo fmt
```

For linting, it uses [clippy](https://github.com/rust-lang/rust-clippy). You can run it with this command:

```shell
$ cargo clippy --all-targets --all-features -- --no-deps -W clippy::pedantic -A clippy::missing_errors_doc -A clippy::missing_panics_doc -A clippy::default_trait_access
```

Or using our defined alias

```shell
$ cargo lint
```

## Spelling

Starknet Foundry uses [typos](https://github.com/marketplace/actions/typos-action) for spelling checks.

You can run the checker with

```shell
$ typos
```

Some typos can be automatically fixed by running

```shell
$ typos -w
```

## Contributing

Read the general contribution guideline [here](https://github.com/foundry-rs/starknet-foundry/blob/master/CONTRIBUTING.md)
