# Installation

Starknet Foundry is easy to install on Linux, Mac and Windows systems.
In this section, we will walk through the process of installing Starknet Foundry.

### Requirements

To use Starknet Foundry, you need [Scarb](https://docs.swmansion.com/scarb/download.html) installed
and added to your `PATH` environment variable.
You can find what version of Scarb is compatible with your version of Starknet Foundry
in [release notes](https://github.com/foundry-rs/starknet-foundry/releases).

### Install via `snfoundryup`

Snfoundryup is the Starknet Foundry toolchain installer. You can find more about it [here](https://github.com/foundry-rs/starknet-foundry/tree/master/scripts).

First install `snfoundryup` by running:

```shell
curl -L https://raw.githubusercontent.com/foundry-rs/starknet-foundry/master/scripts/install.sh | bash
```
Follow the instructions and then run:

```shell
snfoundryup
```

You can also specify a version you wish to install:

```shell
snfoundryup -v 0.9.0
```
See `snfoundryup --help` for more options, like installing from a specific version or commit.

To verify that the Starknet Foundry is installed correctly, run `snforge --version` and `sncast --version`.
## How to build Starknet Foundry from source code

If you are unable to install Starknet Foundry using the instructions above, you can try building it from
the [source code](https://github.com/foundry-rs/starknet-foundry) as follows:

1. [Set up a development environment.](../development/environment-setup.md)
2. Run `cd starknet-foundry && cargo build --release`. This will create a `target` directory.
3. Move the `target` directory to the desired location (e.g. `~/.starknet-foundry`).
4. Add `DESIRED_LOCATION/target/release/` to your `PATH`.
