# Predeployed contracts

`snforge` provides a set of predeployed contracts for use in testing. To support this functionality, we embed their artifacts in the final binary, but we do not keep generated artifacts in repository. Instead, they are generated locally and in CI with `scripts/setup_predeployed_contracts.sh`.

## Generating artifacts for predeployed contracts

To generate artifacts for predeployed contracts, run:

```shell
$ ./scripts/setup_predeployed_contracts.sh
```

Script does the following:
- clones repos (e.g. `starkgate-contracts` for STRK and ETH)
- checks out commit
- enables CASM generation and debug info needed for trace/backtrace support
- builds contracts
- zips generated artifacts and writes `sierra.json.gz` and `casm.json.gz` files into `crates/cheatnet/src/data/predeployed_contracts`

## Modifying predeployed contracts

In order to update existing contracts or add new ones, `scripts/setup_predeployed_contracts.sh` needs to be updated. The script can be structured to source the contract from an external repository or a local directory, depending on the needs. Also, artifacts need to updated/added to `cheatnet/build.rs` in order to be included in the build process.

> 📝 **Note**
>
> When adding a new predeployed contract, make sure the class matches the one deployed on mainnet.

Structure of `predeployed_contracts` directory should be as follows:

```shell
$ tree crates/cheatnet/src/data/predeployed_contracts
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

## Existing predeployed contracts

| Token | Contract Name | Source Code (Cairo) | Class on Mainnet | Commit | Version |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **STRK** | `ERC20Lockable` | [View Source](https://github.com/starknet-io/starkgate-contracts/blob/07e11c39119a10d5742735be5b1d51894ebf5311/packages/strk/src/erc20_lockable.cairo) | [`0x02e7...98fc`](https://voyager.online/class/0x02e77ee61d4df3d988ee1f42ea5442e913862cc82c2584d212ecda76666498fc) | [`07e11c3`](https://github.com/starknet-io/starkgate-contracts/commit/07e11c39119a10d5742735be5b1d51894ebf5311) | v3.0.0 |
| **ETH** | `ERC20Mintable` | [View Source](https://github.com/starknet-io/starkgate-contracts/blob/07e11c39119a10d5742735be5b1d51894ebf5311/packages/sg_token/src/erc20_mintable.cairo) | [`0x00b4...4ec`](https://voyager.online/class/0x00b45dbc3714180381c5680e41931172d67194d77d504413465390e0bef194ec) | [`07e11c3`](https://github.com/starknet-io/starkgate-contracts/commit/07e11c39119a10d5742735be5b1d51894ebf5311) | v3.0.0 |

Steps to update STRK and ETH predeployed contracts:

1. Run:

    ```shell
    ./scripts/setup_predeployed_contracts.sh
    ```

2. By default, the script:
   - clones `starkgate-contracts`
   - checks out commit `07e11c39119a10d5742735be5b1d51894ebf5311`
   - enables CASM generation and debug info needed for trace/backtrace support
   - writes generated `casm.json.gz` and `sierra.json.gz` files into `crates/cheatnet/src/data/predeployed_contracts`

3. If needed, you can override the source repo, revision, source directory, output directory, or `scarb` binary:

    ```shell
    PREDEPLOYED_CONTRACTS_SOURCE_DIR=/path/to/starkgate-contracts \
    PREDEPLOYED_CONTRACTS_REF=07e11c39119a10d5742735be5b1d51894ebf5311 \
    PREDEPLOYED_CONTRACTS_OUTPUT_DIR=/path/to/starknet-foundry/crates/cheatnet/src/data/predeployed_contracts \
    SCARB_BIN=scarb \
    ./scripts/setup_predeployed_contracts.sh
    ```

The generated files are gitignored. CI uses the shared `.github/actions/prepare-predeployed-contracts` action, which restores cached artifacts when possible and falls back to `scripts/setup_predeployed_contracts.sh` when the cache is cold. `snforge` loads the gzipped artifacts at runtime and caches the decompressed Sierra files when debugging metadata is needed.
