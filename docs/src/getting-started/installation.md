# Installation

Starknet-Foundry is easy to install on Linux, Mac and Windows systems. In this section, we will walk through the process of installing Starknet-Foundry.

### Requirements
To use Starknet-Foundry, you need [Scarb](https://docs.swmansion.com/scarb/docs/install) installed and added to your `PATH` environment variable. 

### Install via installation script
1. Open a terminal and run the following command:
```shell
curl -L https://raw.githubusercontent.com/foundry-rs/starknet-foundry/master/install.sh | bash
```
2. Close and reopen the terminal.
3. To check if the Starknet-Foundry is installed correctly, run `forge -v` and `cast -v`.

If you want to specify a version, run the following command with the requested version:

```shell
curl -L https://raw.githubusercontent.com/foundry-rs/starknet-foundry/master/install.sh | bash -s -- -v 0.1.0
```


## How to build Starknet-Foundry from source code
If you are unable to install Starknet-Foundry using the instructions above, you can try building it from the [source code](https://github.com/foundry-rs/starknet-foundry) as follows:

1. [Set up a development environment.](https://github.com/software-mansion/protostar#setting-up-environment)
2. Run `cd starknet-foundry && cargo build --bin --release`. This will create a `starknet-foundry/target` directory.
3. Move the `target` directory to the desired location (e.g. `~/.starknet-foundry`).
4. Add `DESIRED_LOCATION/target/release/` to your `PATH`.
