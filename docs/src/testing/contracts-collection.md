# How Contracts Are Collected

`snforge` supports two mechanism for collecting contracts used in tests.
The default one depends on Scarb version used and can be controlled with `--no-optimization` flag.

- If using Scarb version greater or equal to
  2.8.3, [optimized collection mechanism](contract-collection/new-mechanism.md) is used by default.
- If using Scarb version below 2.8.3 or using `--no-optimization` flag with
  `snforge test` [old collection mechanism](contract-collection/old-mechanism.md) is used.

> 📝 **Note**
>
> When using Scarb versions older than 2.8.3 it is **not possible** to enable new mechanism.
> Migrating to new Scarb version is required.

## Differences Between Collection Mechanisms

| Feature                                                 | Old Mechanism | Optimised Mechanism |
|---------------------------------------------------------|---------------|---------------------|
| Using contracts from `/src`                             | ✅             | ✅                   |
| Using contracts from `/tests`                           | ❌             | ✅                   |
| Using contracts from modules marked with `#[cfg(test)]` | ❌             | ✅                   |
| Using contracts from dependencies                       | ✅             | ✅                   |
| Contracts more closely resemble ones from real network  | ✅             | ❌                   |
| Additional compilation step required (`scarb build`)    | ✅             | ❌                   |
