# `snforge` 0.47.0 Migration Guide

Starting from version 0.47.0, `snforge` will by default support Scarb versions 2.12 or newer.
This is due by migration to the Scarb V2 version of procedural macros, which are used to handle arguments like `#[test]`
in `snforge`.
The benefit of this migration is that tools like Cairo plugin to VSCode will start showing better, more descriptive
errors.

To continue using `snforge` you will need to perform different actions depending on your Scarb version.

## Scarb Versions >= 2.12.0

For Scarb versions >= 2.12.0, we recommend upgrading your `snforge_std` dependency to the one matching your `snforge`
installation (0.47.0 at the time of writing of this doc).

In your `Scarb.toml` file, update the dependency.

```toml
[dev-dependencies]
snforge_std = "0.47.0"
```

No further action is required.

## Scarb Versions < 2.12.0

> 📝 **Note**
>
> We recommend upgrading to at least Scarb 2.12.0, as steps outlined below will stop being supported soon.

For Scarb versions < 2.12.0 it is still possible to continue using the latest `snforge`.
We now publish a new package `snforge_std_compatibility`, supporting versions < 2.12.0.

> 📝 **Note**
>
> `snforge_std_compatibility` may not provide full functionality in comparison to `snforge_std`.

First, in your `Scarb.toml`, remove the `snforge_std` dependency and add `snforge_std_compatibility`.

```diff
[dev-dependencies]
- snforge_std = "0.46.0"
+ snforge_std_compatibility = "0.47.0"
```

Next, replace all kinds of imports in your code from `snforge_std` to `snforge_std_compatibility`

```diff
// Replace use statements
- use snforge_std::{ContractClassTrait, start_cheat_caller_address};
+ use snforge_std_compatibility::{ContractClassTrait, start_cheat_caller_address};

// Replace full path usages
- let result = snforge_std::declare("MyContract").unwrap();
+ let result = snforge_std_compatibility::declare("MyContract").unwrap();
```
