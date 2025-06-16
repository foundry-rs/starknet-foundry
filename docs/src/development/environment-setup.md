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
> If you haven't pushed your branch to the remote yet (you've been working only locally), two tests will fail:
> 
> - `e2e::running::init_new_project_test`
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

<details>
<summary>Output:</summary>

```shell
$ typos -w
```
</details>

## Contributing

Read the general contribution guideline [here](https://github.com/foundry-rs/starknet-foundry/blob/master/CONTRIBUTING.md)
