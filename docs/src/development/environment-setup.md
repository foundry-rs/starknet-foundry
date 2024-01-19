# Environment Setup

> 💡 **Info**
> 
> This setup is for development of Starknet Foundry.
>
> If you don't wish to contribute, you can omit these instructions.

Install the latest stable [Rust](https://www.rust-lang.org/tools/install) version.
If you already have Rust installed make sure to upgrade it by running

```shell
$ rustup update
```

To verify that project was cloned and set up correctly, you can run

```shell
$ cargo check
```

## Running Tests

> 📝 **Note**
> 
> Make sure you run `./scripts/install_devnet.sh`
> and then set [Scarb](https://docs.swmansion.com/scarb/) version 
> [compatible](https://github.com/foundry-rs/starknet-foundry/releases) with both `snforge` and `sncast`
> after setting up the development environment, otherwise the tests will fail.

Tests can be run with:

```shell
$ cargo test
```

> 💡 **Info**
>
> Please make sure you're using scarb installed via asdf - otherwise some tests may fail.
> To verify, run:
> 
> ```shell
> $ which scarb
> $HOME/.asdf/shims/scarb
> ```
> 
> If you previously installed scarb using official installer, you may need to remove this installation or modify your PATH to make sure asdf installed one > is always used.

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
