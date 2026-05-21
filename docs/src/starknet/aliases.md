# Aliases

When you use commands like `sncast call`, `sncast invoke`, etc. you typically need to specify some felt values, for example `--contract-address`:
```shell
$ sncast call --contract-address 0xcd8f9ab31324bb93251837e4efb4223ee195454f6304fcfcb277e277653008 --function get --calldata 0x0 --block-id latest
```

Instead, you can create aliases for felt values (contract addresses, class hashes, etc.) in `snfoundry.toml`.

Then, you can pass them on CLI using the `@alias` syntax:

<!-- TODO(#4225): this would require extended doc test setup (e.g. defining config or emulating aliases using env var) -->
<!-- { "ignored": true } -->
```shell
$ sncast call --contract-address @map --function get --calldata 0x0 --block-id latest
```

## Defining aliases

1. Open your preferred `snfoundry.toml` configuration file. For details on `snfoundry.toml` configuration, see [Configuration](../projects/configuration.md#sncast).
   > 💡 **Tip**
   > If you want your aliases globally available, you'll likely want to edit [global](../projects/configuration.md#global-configuration-file-location) config file, and add aliases to the default profile.
2. Add an `[aliases]` table under your `sncast` profile:
   ```toml
   [sncast.default]
   network = "sepolia"
   
   [sncast.default.aliases]
   map = "0xcd8f9ab31324bb93251837e4efb4223ee195454f6304fcfcb277e277653008"
   map-class = "0x2a09379665a749e609b4a8459c86fe954566a6beeaddd0950e43f6c700ed321"
   ```
   Values are felts in hex or decimal form.

> 💡 **Tip**
> 
> Aliases follow the same rules as other `snfoundry.toml` settings.
> That means you can define aliases in the global and local config files, and create the per-profile aliases, for example:
> ```toml
> [sncast.myprofile]
> network = "devnet"
> 
> [sncast.myprofile.aliases]
> predeployed-account = "0x691a61b12a7105b1372cc377f135213c11e8400a546f6b0e7ea0296046690ce"
> ```
> Aliases follow the same [precedence rules](../projects/configuration.md#interaction-between-local-and-global-profiles) as other `snfoundry.toml` settings: aliases defined in higher-precedence config override lower-precedence ones.


## Using aliases in CLI

Where a command accepts a felt argument that supports aliases, you can pass `@alias` instead of felt literal:

<!-- TODO(#4225): this would require extended doc test setup (e.g. defining config or emulating aliases using env var) -->
<!-- { "ignored": true } -->
```shell
$ sncast call --contract-address @map --function get --calldata 0x0 --block-id latest
```

## Listing aliases

<!-- TODO: add `sncast aliases` command -->
TBD

## `@alias` interaction with multicall `@id`

<!-- TODO: explain @alias_id vs @id precedence -->
TBD

## Supported commands

Currently, aliases are supported for:

- `sncast call --contract-address`
- `sncast invoke --contract-address`


