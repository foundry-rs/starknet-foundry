# Library Functions References

* [`declare`](snforge-library/declare.md) - declares a contract and returns a struct on
  which [`precalculate_address`](snforge-library/precalculate_address.md) and [`deploy`](snforge-library/deploy.md) can be
  called
* [`precalculate_address`](snforge-library/precalculate_address.md) - calculates a contract address that would be returned
  when calling [`deploy`](snforge-library/deploy.md)
* [`deploy`](snforge-library/deploy.md) - deploys a contract and returns its address
* [`print`](snforge-library/print.md) - displays test data
* [`fs`](snforge-library/fs.md) - module containing functions for interacting with the filesystem
* [`env`](snforge-library/env.md) - module containing functions for interacting with the system environment
* [`signature`](snforge-library/signature.md) - module containing struct and trait for creating `ecdsa` signatures

> ℹ️ **Info**
> To use the library functions you need to add `snforge_std` package as a dependency in
> your [`Scarb.toml`](https://docs.swmansion.com/scarb/docs/guides/dependencies.html#adding-a-dependency)
> using appropriate release tag.
>```toml
> [dependencies]
> snforge_std = { git = "https://github.com/foundry-rs/starknet-foundry.git", tag = "v0.12.0" }
> ```
