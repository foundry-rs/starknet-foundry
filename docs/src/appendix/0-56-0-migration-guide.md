# `snforge` 0.56.0 Migration Guide

Starting from version 0.56.0, `snforge` requires Scarb version 2.12.0 or newer.
This is due to the migration to the Scarb V2 version of procedural macros, which are used to handle arguments like `#[test]`
in `snforge`.
Thanks to this migration, tools like the Cairo plugin for VSCode shows better, more descriptive errors.

## Scarb Versions >= 2.12.0

For Scarb versions >= 2.12.0, we recommend upgrading your `snforge_std` dependency to the one matching your `snforge`
installation (0.56.0 at the time of writing this doc).

In your `Scarb.toml` file, update the dependency:

```toml
[dev-dependencies]
snforge_std = "{{snforge_std_version}}"
```

No further action is required.

## Scarb Versions < 2.12.0

> ⚠️ **Warning**
>
> **Support for Scarb versions < 2.12.0 has been removed in `snforge` 0.56.0.**
>
> If you are using Scarb < 2.12.0, you must upgrade to Scarb 2.12.0 or newer.
> The deprecated `snforge_std_deprecated` package and `snforge_scarb_plugin_deprecated` plugin are no longer supported, and no new versions of these will be published.

To upgrade:

First, update your Scarb installation to version 2.12.0 or newer.

Next, in your `Scarb.toml`, remove the `snforge_std_deprecated` dependency and add `snforge_std`:

```diff
[dev-dependencies]
- snforge_std_deprecated = "0.55.0"
+ snforge_std = "{{snforge_std_version}}"
```

Then, replace all kinds of imports in your code from `snforge_std_deprecated` to `snforge_std`:

```diff
// Replace use statements
- use snforge_std_deprecated::{ContractClassTrait, start_cheat_caller_address};
+ use snforge_std::{ContractClassTrait, start_cheat_caller_address};

// Replace full path usages
- let result = snforge_std_deprecated::declare("MyContract").unwrap();
+ let result = snforge_std::declare("MyContract").unwrap();
```
