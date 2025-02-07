# Installation

Starknet Foundry is easy to install on Linux, macOS and Windows.
In this section, we will walk through the process of installing Starknet Foundry.

## Contents

<!-- TOC -->

* [Installation](#installation)
    * [Contents](#contents)
    * [Requirements](#requirements)
    * [Linux and macOS](#linux-and-macos)
        * [Install asdf](#install-asdf)
        * [Install Scarb version >= 2.7.0](#install-scarb-version--270)
        * [(Optional for Scarb >= 2.10.0) Rust Installation](#optional-for-scarb--2100note-rust-installation)
        * [Install Starknet Foundry](#install-starknet-foundry)
    * [Windows](#windows)
        * [Install Scarb version >= 2.7.0](#install-scarb-version--270-1)
        * [(Optional for Scarb >= 2.10.0) Rust Installation](#optional-for-scarb--2100-rust-installation)
        * [Install Universal Sierra Compiler](#install-universal-sierra-compiler)
        * [Install Starknet Foundry](#install-starknet-foundry-1)
    * [Common Errors](#common-errors)
        * [No Version Set (Linux and macOS Only)](#no-version-set-linux-and-macos-only)
        * [Invalid Rust Version](#invalid-rust-version)
            * [Linux and macOS](#linux-and-macos-1)
            * [Windows](#windows-1)
        * [`scarb test` Isn’t Running `snforge`](#scarb-test-isnt-running-snforge)
    * [Universal-Sierra-Compiler update](#universal-sierra-compiler-update)
        * [Linux and macOS](#linux-and-macos-2)
        * [Windows](#windows-2)
    * [How to build Starknet Foundry from source code](#how-to-build-starknet-foundry-from-source-code)

<!-- TOC -->

## Requirements

> 📝 **Note**
>
> Ensure all requirements are installed and follow the required minimum versions.
> Starknet Foundry will not run if not following these requirements.

To use Starknet Foundry, you need:

- [Scarb](https://docs.swmansion.com/scarb/download.html) version >= 2.7.0
- [Universal-Sierra-Compiler](https://github.com/software-mansion/universal-sierra-compiler)
- _(Optional for Scarb >= 2.10.0)_[^note] [Rust](https://www.rust-lang.org/tools/install) version >= 1.80.1

all installed and added to your `PATH` environment variable.

[^note]: Additionally, your platform must be one of the supported:
* aarch64-apple-darwin
* aarch64-unknown-linux-gnu
* x86_64-apple-darwin
* x86_64-pc-windows-msvc
* x86_64-unknown-linux-gnu

> 📝 **Note**
>
> `Universal-Sierra-Compiler` will be automatically installed if you use `asdf` or `snfoundryup`.
> You can also create `UNIVERSAL_SIERRA_COMPILER` env var to make it visible for `snforge`.

## Linux and macOS

> ℹ️ **Info**
>
> If you already have installed Rust, Scarb and asdf simply run
> `asdf plugin add starknet-foundry`

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

Set a version globally (in your ~/.tool-versions file):

```shell
asdf global scarb latest
```

To verify that Scarb was installed, run

```shell
scarb --version
```

and verify that version is >= 2.7.0

### (Optional for Scarb >= 2.10.0)[^note] Rust Installation

> ℹ️️ **Info**
>
> Rust installation is only required if **ANY** of the following is true:
>
> * You are using Scarb version <= 2.10.0
> * Your platform is not one of the following supported platforms:
>   * aarch64-apple-darwin
>   * aarch64-unknown-linux-gnu
>   * x86_64-apple-darwin
>   * x86_64-pc-windows-msvc
>   * x86_64-unknown-linux-gnu

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

To verify that correct Rust version was installed, run

```shell
rustc --version
```

and verify that version is >= 1.80.1

See [Rust docs](https://doc.rust-lang.org/beta/book/ch01-01-installation.html#installation) for more details.

### Install Starknet Foundry

First, add Starknet Foundry plugin to asdf

```shell
asdf plugin add starknet-foundry
```

Install Starknet Foundry

```shell
asdf install starknet-foundry latest
```

Set a version globally (in your ~/.tool-versions file):

```shell
asdf global starknet-foundry latest
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

> 🐧 **Info** - WSL (Windows Subsystem for Linux)
>
> Starknet Foundry can be installed natively on Windows, but currently, for smoother experience, it is recommended to
> use [WSL](https://learn.microsoft.com/en-us/windows/wsl/install).
>
> If you are using WSL, please follow the [Linux and macOS](#linux-and-macos) guide.

### Install Scarb version >= 2.7.0

Follow the instructions from [Scarb docs](https://docs.swmansion.com/scarb/download.html#windows).

1. Download the release archive matching your CPU architecture
   from [https://docs.swmansion.com/scarb/download.html#precompiled-packages](https://docs.swmansion.com/scarb/download.html#precompiled-packages).
2. Extract it to a location where you would like to have Scarb installed. We recommend `%LOCALAPPDATA%\Programs\scarb`.
3. From this directory, get the full path to `scarb\bin` and add it to PATH.
   See [this article](https://www.architectryan.com/2018/03/17/add-to-the-path-on-windows-10/) for instructions on
   Windows 10 and 11.

To verify that Scarb was installed, run

```shell
scarb --version
```

and verify that version is >= 2.7.0

### (Optional for Scarb >= 2.10.0)[^note] Rust Installation

> ℹ️️ **Info**
>
> Rust installation is only required if:
>
> You are using Scarb version <= 2.10.0, *OR*
> * Your platform is not one of the following supported platforms:
>   * x86_64-pc-windows-msvc

Go to [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install) and follow the installation
instructions.

To verify that correct Rust version was installed, run

```shell
rustc --version
```

and verify that version is >= 1.80.1

See [Rust docs](https://doc.rust-lang.org/beta/book/ch01-01-installation.html#installation) for more details.

### Install Universal Sierra Compiler

1. Download the release archive matching your CPU architecture
   from [https://github.com/software-mansion/universal-sierra-compiler/releases/latest](https://github.com/software-mansion/universal-sierra-compiler/releases/latest).
   Look for package with `windows`
   in the name e.g.
   `universal-sierra-compiler-v2.3.0-x86_64-pc-windows-msvc.zip`.
2. Extract it to a location where you would like to have Starknet Foundry installed. We recommend
   `%LOCALAPPDATA%\Programs\universal-sierra-compiler`.
3. From this directory, get the full path to `universal-sierra-compiler\bin` and add it to PATH.
   See [this article](https://www.architectryan.com/2018/03/17/add-to-the-path-on-windows-10/) for instructions on
   Windows 10 and 11.

To verify that Starknet Foundry was installed, run

```shell
universal-sierra-compiler --version
```

### Install Starknet Foundry

1. Download the release archive matching your CPU architecture
   from [https://github.com/foundry-rs/starknet-foundry/releases/latest](https://github.com/foundry-rs/starknet-foundry/releases/latest).
   Look for package with `windows` in the name e.g.
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

### No Version Set (Linux and macOS Only)

Users may encounter this error when trying to use `snforge` or `sncast` without setting a version:

```shell
No version is set for command snforge
Consider adding one of the following versions in your config file at $HOME/.tool_versions
starknet-foundry 0.32.0
```

This error indicates that `Starknet Foundry` version is unset. To resolve it, set the version globally using asdf:

```shell
asdf global starknet-foundry <version>
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
rustc --version
1.80.1
```

To fix, follow the platform specific instructions:

#### Linux and macOS

If the version is incorrect or the error persists, try changing the global version of Rust

```shell
rustup default stable
```

and local version of Rust

```shell
rustup override set stable
```

#### Windows

Follow [Rust installation](https://www.rust-lang.org/tools/install) and ensure correct version of rust was added to
PATH.

### `scarb test` Isn’t Running `snforge`

By default, `scarb test` doesn't use `snforge` to run tests, and it needs to be configured.
Make sure to include this section in `Scarb.toml`

```toml
[scripts]
test = "snforge test"
```

## Universal-Sierra-Compiler update

If you would like to bump the USC manually (e.g. when the new Sierra version is released) you can do it by running:

### Linux and macOS

```shell
curl -L https://raw.githubusercontent.com/software-mansion/universal-sierra-compiler/master/scripts/install.sh | sh
```

### Windows

Follow [Universal Sierra Compiler installation for Windows](#install-universal-sierra-compiler).

## How to build Starknet Foundry from source code

If you are unable to install Starknet Foundry using the instructions above, you can try building it from
the [source code](https://github.com/foundry-rs/starknet-foundry) as follows:

1. [Set up a development environment.](../development/environment-setup.md)
2. Run `cd starknet-foundry && cargo build --release`. This will create a `target` directory.
3. Move the `target` directory to the desired location (e.g. `~/.starknet-foundry`).
4. Add `DESIRED_LOCATION/target/release/` to your `PATH`.
