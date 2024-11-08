# How Contracts Are Collected

`snforge` supports two mechanism for collecting contracts used in tests.
The default depends on Scarb version used and can be controlled with `--no-optimized` flag.

- If using Scarb version greater or equal to 2.8.3, [optimized mechanism](contract-collection/new-mechanism.md) is used
  by default.
- If using Scarb version below 2.8.3 or using `--no-optimized` flag with
  `snforge test` [old, slower mechanism](contract-collection/old-mechanism.md) is used.

> 📝 **Note**
>
> When using Scarb versions older than 2.8.3 it is **not possible** to enable new mechanism.
> Migrating to new Scarb version is required.
