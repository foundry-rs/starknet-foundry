# Library Functions References

* [`declare`](cast-library/declare.md) - declares a contract
* [`deploy`](cast-library/deploy.md) - deploys a contract
* [`invoke`](cast-library/invoke.md) - invokes a contract's function
* [`call`](cast-library/call.md) - calls a contract's function

> ℹ️ **Info**
> To use the library functions you need to add `sncast_std` package as a dependency in
> your [`Scarb.toml`](https://docs.swmansion.com/scarb/docs/guides/dependencies.html#adding-a-dependency)
> using appropriate release tag.
>```toml
> [dependencies]
> sncast_std = { git = "https://github.com/foundry-rs/starknet-foundry.git", tag = "v0.12.0" }
> ```
