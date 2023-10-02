# Library Functions References

* [`declare`](forge-library/declare.md) - declares a contract and returns a struct on
  which [`precalculate_address`](forge-library/precalculate_address.md) and [`deploy`](forge-library/deploy.md) can be
  called
* [`precalculate_address`](forge-library/precalculate_address.md) - calculates a contract address that would be returned
  when calling [`deploy`](forge-library/deploy.md)
* [`deploy`](forge-library/deploy.md) - deploys a contract and returns its address
* [`print`](forge-library/print.md) - displays test data
* [`fs`](forge-library/fs.md) - module containing functions for interacting with the filesystem
* [`env`](forge-library/env.md) - module containing functions for interacting with the system environment

> ℹ️ **Info**
> To use the library functions you need to add `snforge_std` package as a dependency in
> your [`Scarb.toml`](https://docs.swmansion.com/scarb/docs/guides/dependencies.html#adding-a-dependency)
> using appropriate release tag.
>```toml
> [dependencies]
> snforge_std = { git = "https://github.com/foundry-rs/starknet-foundry.git", tag = "v0.5.0" }
> ```
