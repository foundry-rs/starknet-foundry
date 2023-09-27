# Library Functions References

* [`declare`](forge-library/declare.md) - declares a contract and returns a struct on which [`precalculate_address`](forge-library/precalculate_address.md) and [`deploy`](forge-library/deploy.md) can be called
* [`precalculate_address`](forge-library/precalculate_address.md) - calculates a contract address that would be returned when calling [`deploy`](forge-library/deploy.md)
* [`deploy`](forge-library/deploy.md) - deploys a contract and returns its address
* [`print`](forge-library/print.md) - displays test data
* [`read_txt`](forge-library/read_txt.md) - reads and parses plain text file content into an array of felts
* [`parse_txt`](forge-library/parse_txt.md) - parses plain text file content and tries to deserialize it into the specified type
* [`read_json`](forge-library/read_json.md) - reads and parses json file content into an array of felts
* [`parse_json`](forge-library/parse_json.md) - parses json file content and tries to deserialize it into the specified type
* [`env`](forge-library/env.md) - module containing functions for interacting with the system environment

> ℹ️ **Info**
> To use the library functions you need to add `snforge_std` package as a dependency in
> your [`Scarb.toml`](https://docs.swmansion.com/scarb/docs/guides/dependencies.html#adding-a-dependency) 
> using appropriate release tag.
>```toml
> [dependencies]
> snforge_std = { git = "https://github.com/foundry-rs/starknet-foundry.git", tag = "v0.5.0" }
> ```
