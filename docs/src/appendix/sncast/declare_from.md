# `declare-from`
Declare a contract either: 
- from a compiled Sierra file
- by fetching it from another Starknet instance.

The allowed args depend on the chosen contract source:
- **File:** `--sierra-file` (required)
- **Network:** `--class-hash` (required), `--block-id` (optional),  `--source-url` (optional), `--source-network` (optional)

Note: **file** and **network** args are mutually exclusive.

## Required Common Arguments — Passed By CLI or Specified in `snfoundry.toml`

* [`account`](./common.md#--account--a-account_name)

## `--sierra-file <SIERRA_FILE>`
Required in file mode.

Path to the compiled Sierra contract class file (e.g. `target/dev/MyContract_MyContract.contract_class.json`).

## `--class-hash, -g <CLASS_HASH>`
Required in network mode.

Class hash of contract declared on a different network.

## `--url, -u <RPC_URL>`
Optional.

Starknet RPC node url address.

Overrides url from `snfoundry.toml`.

## `--network <NETWORK>`
Optional.

Use predefined network with public provider

Possible values: `mainnet`, `sepolia`, `devnet`.

Overrides network from `snfoundry.toml`.

## `--source-url, -s <RPC_URL>`
Optional.

Starknet RPC node url address of the source network where the contract is already declared.

## `--source-network <NETWORK>`
Optional.

Use predefined network with public provider where the contract is already declared.

Possible values: `mainnet`, `sepolia`, `devnet`.

## `--max-fee, -m <MAX_FEE>`
Optional.

Maximum fee for the `declare` denoted in FRI. Must be greater than zero. If provided, it is not possible to use any of the following fee related flags: `--l1-gas`, `--l1-data-price`, `--l2-gas`, `--l2-gas-price`, `--l1-data-gas`, `--l1-data-gas-price`.

## `--l1-gas <L1_GAS>`
Optional.

Maximum L1 gas for the `declare` transaction. When not used, defaults to auto-estimation.

## ` --l1-gas-price <l1_gas_price>`
Optional.

Maximum L1 gas unit price for the `declare` transaction. When not used, defaults to auto-estimation.

## `--l2-gas <L2_GAS>`
Optional.

Maximum L2 gas for the `declare` transaction. When not used, defaults to auto-estimation.

## `--l2-gas-price <L2_GAS_PRICE>`
Optional.

Maximum L2 gas unit price for the `declare` transaction. When not used, defaults to auto-estimation.

## `--l1-data-gas <L1_DATA_GAS>`
Optional.

Maximum L1 data gas for the `declare` transaction. When not used, defaults to auto-estimation.

## `--l1-data-gas-price <l1_data_gas_price>`
Optional.

Maximum L1 data gas unit price for the `declare` transaction. When not used, defaults to auto-estimation.

## `--tip <TIP>`
Optional.
Conflicts with: [`--estimate-tip`](#--estimate-tip-estimate_tip)

Tip for the transaction. When not provided, defaults to 0 unless [`--estimate-tip`](#--estimate-tip-estimate_tip) is used.

## `--estimate-tip <ESTIMATE_TIP>`
Optional.
Conflicts with: [`--tip`](#--tip-tip)

If passed, an estimated tip will be added to pay for the transaction. The tip is estimated based on the current network conditions and added to the transaction fee.

## `--nonce, -n <NONCE>`
Optional.

Nonce for transaction. If not provided, nonce will be set automatically.

## `--block-id, -b <BLOCK_ID>`
Optional.

Block identifier on which class of declared contract should be fetched.
Possible values: `pre_confirmed`, `latest`, block hash (0x prefixed string), and block number (u64).
`latest` is used as a default value.
