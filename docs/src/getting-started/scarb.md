# Scarb

[Scarb](https://docs.swmansion.com/scarb) is the package manager and build toolchain for Starknet ecosystem.
Those coming from Rust ecosystem will find Scarb very similar to [Cargo](https://doc.rust-lang.org/cargo/).

Starknet Foundry uses [Scarb](https://docs.swmansion.com/scarb) to:
- [manage dependencies](https://docs.swmansion.com/scarb/docs/reference/specifying-dependencies.html)
- [build contracts](https://docs.swmansion.com/scarb/docs/starknet/contract-target.html)

One of the core concepts of Scarb is its [manifest file](https://docs.swmansion.com/scarb/docs/reference/manifest.html) - `Scarb.toml`.
It can be also used to provide [configuration](../projects/configuration.md) for Starknet Foundry.

Last but not least, remember that in order to use Starknet Foundry, you must have Scarb
[installed](https://docs.swmansion.com/scarb/download.html) and added to the `PATH` environment variable.
