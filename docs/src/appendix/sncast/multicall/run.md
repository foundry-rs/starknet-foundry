# `run`

Execute a single multicall transaction containing every call from passed file.

## Required Common Arguments — Passed By CLI or Specified in `snfoundry.toml`

* [`account`](../common.md#--account--a-account_name)

## `--path, -p <PATH>`
Required.

Path to a TOML file with call declarations.

## `--url, -u <RPC_URL>`
Optional.

Starknet RPC node url address.

Overrides url from `snfoundry.toml`.

## `--network <NETWORK>`
Optional.

Use predefined network with public provider

Possible values: `mainnet`, `sepolia`.

## `--max-fee, -m <MAX_FEE>`
Optional.

Maximum fee for the `invoke` transaction in Fri or Wei depending on fee token or transaction version. When not used, defaults to auto-estimation. Must be greater than zero.

## `--l1-gas <L1_GAS>`
Optional.

Maximum L1 gas for the `invoke` transaction. When not used, defaults to auto-estimation. (Only for STRK fee payment)

## ` --l1-gas-price <l1_gas_price>`
Optional.

Maximum L1 gas unit price for the `invoke` transaction paid in Fri. When not used, defaults to auto-estimation. (Only for STRK fee payment)

## `--l2-gas <L2_GAS>`
Optional.

Maximum L2 gas for the `invoke` transaction. When not used, defaults to auto-estimation. (Only for Wei fee payment)

## `--l2-gas-price <L2_GAS_PRICE>`
Optional.

Maximum L2 gas unit price for the `invoke` transaction paid in Fri. When not used, defaults to auto-estimation. (Only for Wei fee payment)

## `--l1-data-gas <L1_DATA_GAS>`
Optional.

Maximum L1 data gas for the `invoke` transaction. When not used, defaults to auto-estimation. (Only for STRK fee payment)

## `--l1-data-gas-price <l1_data_gas_price>`
Optional.

Maximum L1 data gas unit price for the `invoke` transaction paid in Fri. When not used, defaults to auto-estimation. (Only for STRK fee payment)

File example:

```toml
[[call]]
call_type = "deploy"
class_hash = "0x076e94149fc55e7ad9c5fe3b9af570970ae2cf51205f8452f39753e9497fe849"
inputs = []
id = "map_contract"
unique = false

[[call]]
call_type = "invoke"
contract_address = "0x38b7b9507ccf73d79cb42c2cc4e58cf3af1248f342112879bfdf5aa4f606cc9"
function = "put"
inputs = ["0x123", "234"]

[[call]]
call_type = "invoke"
contract_address = "map_contract"
function = "put"
inputs = ["0x123", "234"]

[[call]]
call_type = "deploy"
class_hash = "0x2bb3d35dba2984b3d0cd0901b4e7de5411daff6bff5e072060bcfadbbd257b1"
inputs = ["0x123", "map_contract"]
unique = false
```
