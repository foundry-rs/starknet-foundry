# Installation

Starknet Foundry is easy to install on Linux, Mac and Windows systems. In this section, we will walk through the process of installing Starknet Foundry.

### Requirements
To use Starknet Foundry, you need [Scarb](https://docs.swmansion.com/scarb/docs/install) installed and added to your `PATH` environment variable.
You can find what version of Scarb is compatible with your version of Starknet Foundry in [release notes](https://github.com/foundry-rs/starknet-foundry/releases).

### Install via installation script

TODO: https://github.com/foundry-rs/starknet-foundry/pull/115

## How to build Starknet Foundry from source code
If you are unable to install Starknet Foundry using the instructions above, you can try building it from the [source code](https://github.com/foundry-rs/starknet-foundry) as follows:

1. [Set up a development environment.](https://github.com/software-mansion/protostar#setting-up-environment)
2. Run `cd starknet-foundry && cargo build --bin --release`. This will create a `starknet-foundry/target` directory.
3. Move the `target` directory to the desired location (e.g. `~/.starknet-foundry`).
4. Add `DESIRED_LOCATION/target/release/` to your `PATH`.
