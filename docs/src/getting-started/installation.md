# Installation

Starknet Foundry is easy to install on Linux, macOS and Windows.
In this section, we will walk through the process of installing Starknet Foundry.

## Contents

<!-- TOC -->
* [Installation](#installation)
  * [Contents](#contents)
  * [Requirements](#requirements)
  * [Linux and MacOS](#linux-and-macos)
    * [Install Rust version >= 1.80.1](#install-rust-version--1801)
    * [Install asdf](#install-asdf)
    * [Install Scarb version >= 2.7.0](#install-scarb-version--270)
    * [Install Starknet Foundry](#install-starknet-foundry)
  * [Windows](#windows)
    * [Install Rust version >= 1.80.1](#install-rust-version--1801-1)
    * [Install Scarb version >= 2.7.0](#install-scarb-version--270-1)
    * [Install Starknet Foundry](#install-starknet-foundry-1)
  * [Common Errors](#common-errors)
    * [No Version Set](#no-version-set)
    * [Invalid Rust Version](#invalid-rust-version)
  * [Universal-Sierra-Compiler update](#universal-sierra-compiler-update)
  * [How to build Starknet Foundry from source code](#how-to-build-starknet-foundry-from-source-code)
<!-- TOC -->

## Requirements

> 📝 **Note**
> Ensure all requirements are installed and follow the required minimum versions.
> Starknet Foundry will not run if not following these requirements.

To use Starknet Foundry, you need:

- [Scarb](https://docs.swmansion.com/scarb/download.html) version >= 2.7.0
- [Universal-Sierra-Compiler](https://github.com/software-mansion/universal-sierra-compiler)
- [Rust](https://www.rust-lang.org/tools/install) version >= 1.80.1

all installed and added to your `PATH` environment variable.

> 📝 **Note**
>
> `Universal-Sierra-Compiler` will be automatically installed if you use `asdf` or `snfoundryup`.
> You can also create `UNIVERSAL_SIERRA_COMPILER` env var to make it visible for `snforge`.

## Linux and MacOS

### Install Rust version >= 1.80.1

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

To verify that correct Rust version was installed, run

```shell
rustc --version
```

and verify that version is >= 1.80.1

See [Rust docs](https://doc.rust-lang.org/beta/book/ch01-01-installation.html#installation) for more details.

### Install asdf

Follow the instructions from [asdf docs](https://asdf-vm.com/guide/getting-started.html#getting-started).

To verify that asdf was installed, run

```shell
asdf --version
```

### Install Scarb version >= 2.7.0

First, add Scarb plugin to asdf

```shell
asdf plugin add scarb
```

Install Scarb

```shell
asdf install scarb latest
```

To verify that Scarb was installed, run

```shell
scarb --version
```

and verify that version is >= 2.7.0

### Install Starknet Foundry

First, add Starknet Foundry plugin to asdf

```shell
asdf plugin add starknet-foundry
```

Install Starknet Foundry

```shell
asdf install starknet-foundry latest
```

To verify that Starknet Foundry was installed, run

```shell
snforge --version
```

or

```shell
sncast --version
```

## Windows

### Install Rust version >= 1.80.1

Go to https://www.rust-lang.org/tools/install and follow the installation instructions.

To verify that correct Rust version was installed, run

```shell
rustc --version
```

and verify that version is >= 1.80.1

See [Rust docs](https://doc.rust-lang.org/beta/book/ch01-01-installation.html#installation) for more details.

### Install Scarb version >= 2.7.0

Follow the instructions from [Scarb docs](https://docs.swmansion.com/scarb/download.html#windows).

1. Download the release archive matching your CPU architecture
   from https://docs.swmansion.com/scarb/download.html#precompiled-packages.
2. Extract it to a location where you would like to have Scarb installed. We recommend `%LOCALAPPDATA%\Programs\scarb`.
3. From this directory, get the full path to `scarb\bin` and add it to PATH.
   See [this article](https://www.architectryan.com/2018/03/17/add-to-the-path-on-windows-10/) for instructions on
   Windows 10 and 11.

To verify that Scarb was installed, run

```shell
scarb --version
```

and verify that version is >= 2.7.0

### Install Starknet Foundry

1. Download the release archive matching your CPU architecture
   from https://github.com/foundry-rs/starknet-foundry/releases/latest. Look for package with `windows` in the name e.g.
   `starknet-foundry-v0.34.0-x86_64-pc-windows-msvc.zip`.
2. Extract it to a location where you would like to have Starknet Foundry installed. We recommend
   `%LOCALAPPDATA%\Programs\snfoundry`.
3. From this directory, get the full path to `snfoundry\bin` and add it to PATH.
   See [this article](https://www.architectryan.com/2018/03/17/add-to-the-path-on-windows-10/) for instructions on
   Windows 10 and 11.

To verify that Starknet Foundry was installed, run

```shell
snforge --version
```

or

```shell
sncast --version
```

## Common Errors

### No Version Set

Users may encounter this error when trying to use `snforge` or `sncast` without setting a version:

```shell
No version is set for command snforge
Consider adding one of the following versions in your config file at $HOME/.tool_versions
starknet-foundry 0.32.0
```

This error indicates that `Starknet Foundry` version is unset. To resolve it, set the version globally using asdf:

```shell
$ asdf global starknet-foundry <version>
```

For additional information on asdf version management, see
the [asdf](https://asdf-vm.com/guide/getting-started.html#_6-set-a-version)

### Invalid Rust Version

When running any `snforge` command, error similar to this is displayed

```shell
Compiling snforge_scarb_plugin v0.34.0
error: package snforge_scarb_plugin v0.34.0 cannot be built because it requires rustc 1.80.1 or newer, while the currently active rustc version is 1.76.0
```

This indicates incorrect Rust version is installed or set.

Verify if rust version >= 1.80.1 is installed

```shell
$ rustc --version
1.80.1
```

If the version is incorrect or the error persists, try changing the global version of Rust

```shell
$ rustup default stable
```

and local version of Rust

```shell
$ rustup override set stable
```

## Universal-Sierra-Compiler update

If you would like to bump the USC manually (e.g. when the new Sierra version is released) you can do it by running:

```shell
$ curl -L https://raw.githubusercontent.com/software-mansion/universal-sierra-compiler/master/scripts/install.sh | sh
```

## How to build Starknet Foundry from source code

If you are unable to install Starknet Foundry using the instructions above, you can try building it from
the [source code](https://github.com/foundry-rs/starknet-foundry) as follows:

1. [Set up a development environment.](../development/environment-setup.md)
2. Run `cd starknet-foundry && cargo build --release`. This will create a `target` directory.
3. Move the `target` directory to the desired location (e.g. `~/.starknet-foundry`).
4. Add `DESIRED_LOCATION/target/release/` to your `PATH`.
