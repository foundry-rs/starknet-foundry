# Testing Smart Contracts

> ℹ️ **Info**
>
> To use the library functions designed for testing smart contracts,
> you need to add `snforge_std` package as a dependency in
> your [`Scarb.toml`](https://docs.swmansion.com/scarb/docs/guides/dependencies.html#development-dependencies)
> using the appropriate version.
>```toml
> [dev-dependencies]
> snforge_std = "0.44.0"
> ```

When writing smart contracts, you often want to test their interactions with the blockchain state and with other
contracts.
This chapter shows how to test smart contracts using Starknet Foundry.
