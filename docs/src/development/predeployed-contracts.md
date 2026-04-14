# Predpeloyed contracts

`snforge` provides a set of predeployed contracts that can be used in the tests. This feature requires us to store CASM of these contracts in our codebase.
Since contracts may receive updates over time, we need to update these files. Below is a list of all predeployed contracts with necessary information to keep the up to date.

## STRK and ETH 

Contracts are taken from the [`starkgate-contracts`](https://github.com/starknet-io/starkgate-contracts) repository. Currently `snforge` uses version [`v3.0.0`](https://github.com/starknet-io/starkgate-contracts/commit/07e11c39119a10d5742735be5b1d51894ebf5311).

Links to contracts:
- STRK: [`ERC20Lockable`](https://github.com/starknet-io/starkgate-contracts/blob/07e11c39119a10d5742735be5b1d51894ebf5311/packages/strk/src/erc20_lockable.cairo)
- ETH : [`ERC20Mintable`](https://github.com/starknet-io/starkgate-contracts/blob/07e11c39119a10d5742735be5b1d51894ebf5311/packages/sg_token/src/erc20_mintable.cairo)


Steps to obtain CASM for these contracts:

1. Clone `starkgate-contracts` repository and checkout `v3.0.0` tag.

```shell
git clone https://github.com/starknet-io/starkgate-contracts
cd starkgate-contracts
git checkout v3.0.0
```

2. Enable CASM generation by adding the following lines to `Scarb.toml`.
This should be done in `sg_token` and `strk` packages.

```toml
...

[[target.starknet-contract]]
casm = true

...
```

3. Compile contracts with `scarb`

```shell
scarb --release build
```

4. Visit `target/release` directory and copy relevant CASM files.
