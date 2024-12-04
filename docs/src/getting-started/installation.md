# Installation

Starknet Foundry is easy to install on Linux, macOS and Windows.
In this section, we will walk through the process of installing Starknet Foundry.

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

## Installation on Linux and macOS

### Installation via [asdf](https://asdf-vm.com/)

First, add the Starknet Foundry plugin to asdf:

```shell
$ asdf plugin add starknet-foundry
```

### Install via `snfoundryup`

Snfoundryup is the Starknet Foundry toolchain installer.

You can install it by running:

```shell
$ curl -L https://raw.githubusercontent.com/foundry-rs/starknet-foundry/master/scripts/install.sh | sh
```

Follow the instructions and then run:

```shell
$ snfoundryup
```

See `snfoundryup --help` for more options.

To verify that the Starknet Foundry is installed correctly, run `snforge --version` and `sncast --version`.

## Installation on Windows

As for now, Starknet Foundry on Windows needs manual installation, but necessary steps are kept to minimum:

1. [Download the release](https://github.com/foundry-rs/starknet-foundry/releases) archive matching your CPU
   architecture.
2. Extract it to a location where you would like to have Starknet Foundry installed. A folder named snfoundry in
   your [
   `%LOCALAPPDATA%\Programs`](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid?redirectedfrom=MSDN#FOLDERID_UserProgramFiles)
   directory will suffice:

```batch
%LOCALAPPDATA%\Programs\snfoundry
```

3. Add path to the snfoundry\bin directory to your PATH environment variable.
4. Verify installation by running the following command in new terminal session:

   ```shell
   $ snforge --version
   ```

   and

   ```
   $ sncast --version
   ```

## Universal-Sierra-Compiler update

If you would like to bump the USC manually (e.g. when the new Sierra version is released) you can do it by running:

```shell
$ curl -L https://raw.githubusercontent.com/software-mansion/universal-sierra-compiler/master/scripts/install.sh | sh
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
error: package `snforge_scarb_plugin v0.34.0 cannot be built because it requires rustc 1.80.1 or newer, while the currently active rustc version is 1.76.0
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

## How to build Starknet Foundry from source code

If you are unable to install Starknet Foundry using the instructions above, you can try building it from
the [source code](https://github.com/foundry-rs/starknet-foundry) as follows:

1. [Set up a development environment.](../development/environment-setup.md)
2. Run `cd starknet-foundry && cargo build --release`. This will create a `target` directory.
3. Move the `target` directory to the desired location (e.g. `~/.starknet-foundry`).
4. Add `DESIRED_LOCATION/target/release/` to your `PATH`.
