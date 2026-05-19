# Predeployed contracts

`snforge` provides a set of predeployed contracts for use in testing. Their artifacts are generated from pinned upstream
sources with `scripts/setup_predeployed_contracts.sh`.

The generated `casm.json` and `sierra.json` files are tracked in git. During build, `cheatnet` gzips those committed
artifacts into `OUT_DIR` before embedding them in the final binary.

To refresh the committed artifacts after changing source revisions or generation logic, run:

```shell
$ ./scripts/setup_predeployed_contracts.sh
```

## List of predeployed contracts

| Token | Contract Name | Source Code (Cairo) | Class on Mainnet | Commit Hash | Version |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **STRK** | `ERC20Lockable` | [View Source](https://github.com/starknet-io/starkgate-contracts/blob/07e11c39119a10d5742735be5b1d51894ebf5311/packages/strk/src/erc20_lockable.cairo) | [`0x02e7...98fc`](https://voyager.online/class/0x02e77ee61d4df3d988ee1f42ea5442e913862cc82c2584d212ecda76666498fc) | [`07e11c3`](https://github.com/starknet-io/starkgate-contracts/commit/07e11c39119a10d5742735be5b1d51894ebf5311) | `v3.0.0` |
| **ETH** | `ERC20Mintable` | [View Source](https://github.com/starknet-io/starkgate-contracts/blob/07e11c39119a10d5742735be5b1d51894ebf5311/packages/sg_token/src/erc20_mintable.cairo) | [`0x00b4...4ec`](https://voyager.online/class/0x00b45dbc3714180381c5680e41931172d67194d77d504413465390e0bef194ec) | [`07e11c3`](https://github.com/starknet-io/starkgate-contracts/commit/07e11c39119a10d5742735be5b1d51894ebf5311) | `v3.0.0` |

## Updating predeployed contracts

To add a new predeployed contract or update an existing one, extend the generator script so that it writes `casm.json` and `sierra.json` into a
new subdirectory under `crates/cheatnet/src/data/predeployed_contracts`.

After regenerating the artifacts, commit the updated files. CI rebuilds them from source and fails if the committed
versions are stale.

> 📝 **Note**
>
> When adding a new predeployed contract, make sure the class matches the one deployed on mainnet.

Structure of `predeployed_contracts` directory should be as follows:

```shell
$ tree crates/cheatnet/src/data/predeployed_contracts
crates/cheatnet/src/data/predeployed_contracts
├── ERC20Lockable
│   ├── casm.json
│   └── sierra.json
└── ERC20Mintable
    ├── casm.json
    └── sierra.json
```
