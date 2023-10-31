# Environment Setup

Install the latest stable [Rust](https://www.rust-lang.org/tools/install) version.
If you already have Rust installed make sure to upgrade it by running

```shell
$ rustup update
```

To verify that project was cloned and set up correctly, you can run

```shell
$ cargo check
```

## External Dependencies

To run Starknet Foundry tests, you must install these tools on your computer:

- [asdf](https://asdf-vm.com/guide/getting-started.html)
- [starknet-devnet](https://0xspaceshard.github.io/starknet-devnet/docs/intro)

It is not possible to run tests without these installed.

## Running Tests

> ⚠️ Make sure you run `./scripts/install_devnet.sh`
> and then set [Scarb](https://docs.swmansion.com/scarb/) version 
> [compatible](https://github.com/foundry-rs/starknet-foundry/releases) with both `snforge` and `sncast`
> after setting up the development environment, otherwise the tests will fail.

Tests can be run with:

```shell
$ cargo test
```

## Formatting and Lints

Starknet Foundry uses [rustfmt](https://github.com/rust-lang/rustfmt) for formatting. You can run the formatter with

```shell
$ cargo fmt
```

For linting, it uses [clippy](https://github.com/rust-lang/rust-clippy). You can run it with this command:

```shell
$ cargo clippy --all-targets --all-features -- --no-deps -W clippy::pedantic -A clippy::missing_errors_doc -A clippy::missing_panics_doc -A clippy::default_trait_acces
```

Or using our defined alias

```shell
$ cargo lint
```
