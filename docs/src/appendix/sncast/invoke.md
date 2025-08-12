# `invoke`
Send an invoke transaction to Starknet.

## Required Common Arguments — Passed By CLI or Specified in `snfoundry.toml`

* [`account`](./common.md#--account--a-account_name)

## `--contract-address, -d <CONTRACT_ADDRESS>`
Required.

The address of the contract being called in hex (prefixed with '0x') or decimal representation.

## `--function, -f <FUNCTION_NAME>`
Required.

The name of the function to call.

## `--calldata, -c <CALLDATA>`
Optional.
Conflicts with: [`--arguments`](#--arguments)

Inputs to the function, represented by a list of space-delimited values `0x1 2 0x3`.
Calldata arguments may be either 0x hex or decimal felts.

## `--arguments`
Optional.
Conflicts with: [`--calldata`](#--calldata--c-calldata)

Function arguments provided as a comma-separated string of Cairo expressions.
For example: `--arguments '1, 2, MyStruct { x: 1, y: 2 }, MyEnum::Variant'`

For more information on supported expressions and syntax, see [Calldata Transformation](../../starknet/calldata-transformation.md).

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

Maximum fee for the `invoke` denoted in FRI. Must be greater than zero. If provided, it is not possible to use any of the following fee related flags: `--l1-gas`, `--l1-data-price`, `--l2-gas`, `--l2-gas-price`, `--l1-data-gas`, `--l1-data-gas-price`.

## `--l1-gas <L1_GAS>`
Optional.

Maximum L1 gas for the `invoke` transaction. When not used, defaults to auto-estimation.

## ` --l1-gas-price <l1_gas_price>`
Optional.

Maximum L1 gas unit price for the `invoke` transaction. When not used, defaults to auto-estimation.

## `--l2-gas <L2_GAS>`
Optional.

Maximum L2 gas for the `invoke` transaction. When not used, defaults to auto-estimation.

## `--l2-gas-price <L2_GAS_PRICE>`
Optional.

Maximum L2 gas unit price for the `invoke` transaction. When not used, defaults to auto-estimation.

## `--l1-data-gas <L1_DATA_GAS>`
Optional.

Maximum L1 data gas for the `invoke` transaction. When not used, defaults to auto-estimation.

## `--l1-data-gas-price <l1_data_gas_price>`
Optional.

Maximum L1 data gas unit price for the `invoke` transaction. When not used, defaults to auto-estimation.

## `--tip <TIP>`
Optional.

Tip for the transaction. When not used, defaults to 0.

## `--nonce, -n <NONCE>`
Optional.

Nonce for transaction. If not provided, nonce will be set automatically.
