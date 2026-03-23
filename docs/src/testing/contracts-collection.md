# How Contracts Are Collected

`snforge` supports two mechanisms for collecting contracts used in tests.

- By default, [optimized collection mechanism](contract-collection/new-mechanism.md) is used.
- If running `snforge test` with `--no-optimization` flag, the [old collection mechanism](contract-collection/old-mechanism.md) is used.

## Differences Between Collection Mechanisms

| Feature                                                 | Old Mechanism | Optimised Mechanism |
|---------------------------------------------------------|---------------|---------------------|
| Using contracts from `/src`                             | ✅             | ✅                   |
| Using contracts from `/tests`                           | ❌             | ✅                   |
| Using contracts from modules marked with `#[cfg(test)]` | ❌             | ✅                   |
| Using contracts from dependencies                       | ✅             | ✅                   |
| Contracts more closely resemble ones from real network  | ✅             | ❌                   |
| Less compilation steps required (faster compilation)    | ❌             | ✅                   |
| Additional compilation step required (`scarb build`)    | ✅             | ❌                   |
