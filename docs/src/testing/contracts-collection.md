# How Contracts Are Collected

`snforge` supports two mechanism for collecting contracts used in tests.
The default one depends on Scarb version used and can be controlled with `--no-optimization` flag.

- If using Scarb version >= 2.8.3, [optimized collection mechanism](contract-collection/new-mechanism.md) is used by default.
- If using Scarb version < 2.8.3 or running `snforge test` with `--no-optimization` flag, the [old collection mechanism](contract-collection/old-mechanism.md) is used.

> 📝 **Note**
>
> Enabling new mechanism **requires** Scarb version >= 2.8.3.

## Differences Between Collection Mechanisms

| Feature                                                 | Old Mechanism | Optimised Mechanism |
|---------------------------------------------------------|---------------|---------------------|
| Using contracts from `/src`                             | ✅             | ✅                   |
| Using contracts from `/tests`                           | ❌             | ✅                   |
| Using contracts from modules marked with `#[cfg(test)]` | ❌             | ✅                   |
| Using contracts from dependencies                       | ✅             | ✅                   |
| Contracts more closely resemble ones from real network  | ✅             | ❌                   |
| Additional compilation step required (`scarb build`)    | ✅             | ❌                   |
