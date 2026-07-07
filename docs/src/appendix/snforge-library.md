# Library Reference

> ℹ️ **Info**
> Full documentation for the `snforge` library can be found [here](https://foundry-rs.github.io/starknet-foundry/snforge_std/).

* [declare](appendix/snforge-library/declare.md)
* [declare!](appendix/snforge-library/declare_macro.md)
* [declare_result](appendix/snforge-library/declare_result.md)
* [contract_class](appendix/snforge-library/contract_class.md)
* [`get_call_trace`](snforge-library/get_call_trace.md) - gets current test call trace (with contracts interactions
  included)
* [`fs`](snforge-library/fs.md) - module containing functions for interacting with the filesystem
* [`env`](snforge-library/env.md) - module containing functions for interacting with the system environment
* [`signature`](snforge-library/signature.md) - module containing struct and trait for creating `ecdsa` signatures

> ℹ️ **Info**
> To use cheatcodes you need to add `snforge_std` package as a development dependency in
> your [`Scarb.toml`](https://docs.swmansion.com/scarb/docs/guides/dependencies.html#development-dependencies)
> using the appropriate version.
> ```toml
> [dev-dependencies]
> snforge_std = "{{snforge_std_version}}"
> ```
