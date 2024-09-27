# Scarb

[Scarb](https://docs.swmansion.com/scarb) is the package manager and build toolchain for Starknet ecosystem.
Those coming from Rust ecosystem will find Scarb very similar to [Cargo](https://doc.rust-lang.org/cargo/).

Starknet Foundry uses [Scarb](https://docs.swmansion.com/scarb) to:
- [manage dependencies](https://docs.swmansion.com/scarb/docs/reference/specifying-dependencies.html)
- [build contracts](https://docs.swmansion.com/scarb/docs/extensions/starknet/contract-target.html)

One of the core concepts of Scarb is its [manifest file](https://docs.swmansion.com/scarb/docs/reference/manifest.html) - `Scarb.toml`.
It can be also used to provide [configuration](../projects/configuration.md) for Starknet Foundry Forge.
Moreover, you can modify behaviour of `scarb test` to run `snforge test` as 
described [here](https://docs.swmansion.com/scarb/docs/extensions/testing.html#using-third-party-test-runners).

> 📝 **Note**
> 
>`Scarb.toml` is specifically designed for configuring scarb packages and, by extension, is suitable for `snforge` configurations, 
> which are package-specific. On the other hand, `sncast` can operate independently of scarb workspaces/packages 
> and therefore utilizes a different configuration file, `snfoundry.toml`. This distinction ensures that configurations 
> are appropriately aligned with their respective tools' operational contexts.

Last but not least, remember that in order to use Starknet Foundry, you must have Scarb
[installed](https://docs.swmansion.com/scarb/download.html) and added to the `PATH` environment variable.


### `assert_macros`
> ⚠️ **Recommended only for development** ️⚠️
> 
>Assert macros package provides a set of macros that can be used to write assertions.
In order to use it, your project must have the `assert_macros` dependency added to the `Scarb.toml` file.
These macros are very expensive to run on Starknet, they generate huge amount of steps and are not recommended for production use. 
They are only meant to be used in tests.
For snforge `v0.31.0` and later, this dependency is added by default. But for earlier versions, you need to add it manually.

```toml
[dev-dependencies]
snforge_std = ...
assert_macros = "<scarb-version>"
```

Available assert macros are 
- `assert_eq!`
- `assert_ne!`
- `assert_lt!`
- `assert_le!`
- `assert_gt!`
- `assert_ge!`