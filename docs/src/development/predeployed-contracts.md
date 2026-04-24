# Predeployed contracts

`snforge` provides a set of predeployed contracts for use in testing. To support this functionality, we maintain their artifacts directly in the codebase. The embedded artifacts are stored in gzipped form to reduce the size of the final binary.

## Adding new predeployed contract

To add a new predeployed contract, add a new subdirectory with the name of the contract to `crates/cheatnet/src/data/predeployed_contracts`. Then add gzipped artifact files to this subdirectory: `casm.json.gz` and `sierra.json.gz`.

> 📝 **Note**
>
> When adding a new predeployed contract, make sure the class matches the one deployed on mainnet.

Structure of `predeployed_contracts` directory should be as follows:

```shell
$ tree
.
├── ERC20Lockable
│   ├── casm.json.gz
│   └── sierra.json.gz
├── ERC20Mintable
│   ├── casm.json.gz
│   └── sierra.json.gz
└── <Other contract>
    ├── casm.json.gz
    └── sierra.json.gz
```

## Updating existing predeployed contracts

### STRK and ETH 

These contracts are sourced from the [starkgate-contracts](https://github.com/starknet-io/starkgate-contracts) repository.

**Current Build Configuration:**
- **Version:** `v3.0.0`
- **Commit Hash:** [`07e11c3`](https://github.com/starknet-io/starkgate-contracts/commit/07e11c39119a10d5742735be5b1d51894ebf5311)

| Token | Contract Name | Source Code (Cairo) | Class on Mainnet |
| :--- | :--- | :--- | :--- |
| **STRK** | `ERC20Lockable` | [View Source](https://github.com/starknet-io/starkgate-contracts/blob/07e11c39119a10d5742735be5b1d51894ebf5311/packages/strk/src/erc20_lockable.cairo) | [`0x02e7...98fc`](https://voyager.online/class/0x02e77ee61d4df3d988ee1f42ea5442e913862cc82c2584d212ecda76666498fc) |
| **ETH** | `ERC20Mintable` | [View Source](https://github.com/starknet-io/starkgate-contracts/blob/07e11c39119a10d5742735be5b1d51894ebf5311/packages/sg_token/src/erc20_mintable.cairo) | [`0x00b4...4ec`](https://voyager.online/class/0x00b45dbc3714180381c5680e41931172d67194d77d504413465390e0bef194ec) |

Steps to update STRK and ETH predeployed contracts:

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

3. Adjust compiler configuration in workspace `Scarb.toml`:

    ```toml
    ...

    [profile.release.cairo]
    add-statements-code-locations-debug-info = true
    add-statements-functions-debug-info = true
    panic-backtrace = true

    ...
    ```

4. Compile contracts with `scarb`

    ```shell
    scarb --release build
    ```

5. Visit `target/release` directory and compress the generated artifacts directly into the appropriate `predeployed_contracts` subdirectories in the `cheatnet` codebase:

    ```shell
    gzip -9 -c target/release/strk_ERC20Lockable.compiled_contract_class.json > /path/to/starknet-foundry/crates/cheatnet/src/data/predeployed_contracts/ERC20Lockable/casm.json.gz
    gzip -9 -c target/release/strk_ERC20Lockable.contract_class.json > /path/to/starknet-foundry/crates/cheatnet/src/data/predeployed_contracts/ERC20Lockable/sierra.json.gz
    gzip -9 -c target/release/sg_token_ERC20Mintable.compiled_contract_class.json > /path/to/starknet-foundry/crates/cheatnet/src/data/predeployed_contracts/ERC20Mintable/casm.json.gz
    gzip -9 -c target/release/sg_token_ERC20Mintable.contract_class.json > /path/to/starknet-foundry/crates/cheatnet/src/data/predeployed_contracts/ERC20Mintable/sierra.json.gz
    ```

`snforge` loads the gzipped artifacts at runtime and caches the decompressed Sierra files when debugging metadata is needed.
