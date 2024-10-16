# Library Reference

> ℹ️ **Info**
> Full documentation for the `sncast` library can be found [here](https://delevoxdg.github.io/starknet-foundry/sncast_std/).

* [`declare`](sncast-library/declare.md) - declares a contract
* [`deploy`](sncast-library/deploy.md) - deploys a contract
* [`invoke`](sncast-library/invoke.md) - invokes a contract's function
* [`call`](sncast-library/call.md) - calls a contract's function
* [`get_nonce`](sncast-library/get_nonce.md) - gets account's nonce for a given block tag
* [`tx_status`](sncast-library/tx_status.md) - gets the status of a transaction using its hash
* [`errors`](sncast-library/errors.md) - sncast_std error types reference

> ℹ️ **Info**
> To use the library functions you need to add `sncast_std` package as a dependency in
> your [`Scarb.toml`](https://docs.swmansion.com/scarb/docs/guides/dependencies.html#adding-a-dependency)
> using appropriate release tag.
>```toml
> [dependencies]
> sncast_std = { git = "https://github.com/foundry-rs/starknet-foundry.git", tag = "v0.22.0" }
> ```
