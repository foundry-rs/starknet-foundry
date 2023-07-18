# Environment Setup

Install the latest [Rust](https://www.rust-lang.org/tools/install) version.
If you already have Rust installed make sure to upgrade it by running

```shell
$ rustup update
```

To verify that project was cloned and setup correctly you can run

```shell
$ cargo check
```

## Running Tests

Test scripts require you to have asdf installed. Check out [asdf docs](https://asdf-vm.com/guide/getting-started.html)
for more details.

> ⚠️ Make sure you run `./scripts/prepare-for-tests.sh` after setting up the development environment, otherwise tests
> will fail.

Tests can be run with:

```shell
$ cargo test
```

## Formatting and Lints

Starknet Foundry uses [rustfmt](https://github.com/rust-lang/rustfmt) for formatting. You can run the formatter with

```shell
cargo fmt
```

For linting it uses [clippy](https://github.com/rust-lang/rust-clippy). You can run it with this command:

```shell
$ cargo clippy --all-targets --all-features -- --no-deps -W clippy::pedantic -A clippy::missing_errors_doc -A clippy::missing_panics_doc -A clippy::default_trait_acces
```

Or using our defined alias

```shell
$ cargo lint
```
