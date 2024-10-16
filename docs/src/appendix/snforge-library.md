# Library Reference

> ℹ️ **Info**
> Full documentation for the `snforge` library can be found [here](https://delevoxdg.github.io/starknet-foundry/snforge_std/).

* [`declare`](snforge-library/declare.md) - declares a contract and returns
  a [`ContractClass`](snforge-library/contract_class.md) which can be interacted with later
* [`get_call_trace`](snforge-library/get_call_trace.md) - gets current test call trace (with contracts interactions
  included)
* [`fs`](snforge-library/fs.md) - module containing functions for interacting with the filesystem
* [`env`](snforge-library/env.md) - module containing functions for interacting with the system environment
* [`signature`](snforge-library/signature.md) - module containing struct and trait for creating `ecdsa` signatures

> ℹ️ **Info**
> To use cheatcodes you need to add `snforge_std` package as a development dependency in
> your [`Scarb.toml`](https://docs.swmansion.com/scarb/docs/guides/dependencies.html#development-dependencies)
> using appropriate release tag.
> ```toml
> [dev-dependencies]
> snforge_std = { git = "https://github.com/foundry-rs/starknet-foundry.git", tag = "v0.12.0" }
> ```
