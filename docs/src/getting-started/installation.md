# Installation

Starknet Foundry is easy to install on Linux, Mac and Windows systems.
In this section, we will walk through the process of installing Starknet Foundry.

### Requirements

To use Starknet Foundry, you need:

- [Scarb](https://docs.swmansion.com/scarb/download.html)
- [Universal-Sierra-Compiler](https://github.com/software-mansion/universal-sierra-compiler)

both installed and added to your `PATH` environment variable.

> 📝 **Note**
>
> `Universal-Sierra-Compiler` will be automatically installed if you use `snfoundryup` or `asdf`.
> You can also create `UNIVERSAL_SIERRA_COMPILER` env var to make it visible for `snforge`.

### Install via `snfoundryup`

Snfoundryup is the Starknet Foundry toolchain installer.

You can install it by running:

```shell
curl -L https://raw.githubusercontent.com/foundry-rs/starknet-foundry/master/scripts/install.sh | sh
```

Follow the instructions and then run:

```shell
snfoundryup
```

See `snfoundryup --help` for more options.

To verify that the Starknet Foundry is installed correctly, run `snforge --version` and `sncast --version`.

### Installation via [asdf](https://asdf-vm.com/)

First, add the Starknet Foundry plugin to asdf:

```shell
asdf plugin add starknet-foundry
```

Install the latest version:

```shell
asdf install starknet-foundry latest
```

See [asdf guide](https://asdf-vm.com/guide/getting-started.html) for more details.

### Installation on Windows

As for now, Starknet Foundry on Windows needs manual installation, but necessary steps are kept to minimum:

1. [Download the release](https://github.com/foundry-rs/starknet-foundry/releases) archive matching your CPU
   architecture.
2. Extract it to a location where you would like to have Starknet Foundry installed. A folder named snfoundry in
   your [`%LOCALAPPDATA%\Programs`](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid?redirectedfrom=MSDN#FOLDERID_UserProgramFiles)
   directory will suffice:

```batch
%LOCALAPPDATA%\Programs\snfoundry
```

3. Add path to the snfoundry\bin directory to your PATH environment variable.
4. Verify installation by running the following command in new terminal session:

```shell
snforge --version
sncast --version
```

### Universal-Sierra-Compiler update

If you would like to bump the USC manually (e.g. when the new Sierra version is released) you can do it by running:

```shell
curl -L https://raw.githubusercontent.com/software-mansion/universal-sierra-compiler/master/scripts/install.sh | sh
```

## How to build Starknet Foundry from source code

If you are unable to install Starknet Foundry using the instructions above, you can try building it from
the [source code](https://github.com/foundry-rs/starknet-foundry) as follows:

1. [Set up a development environment.](../development/environment-setup.md)
2. Run `cd starknet-foundry && cargo build --release`. This will create a `target` directory.
3. Move the `target` directory to the desired location (e.g. `~/.starknet-foundry`).
4. Add `DESIRED_LOCATION/target/release/` to your `PATH`.
