# `deploy`
Deploy a contract to Starknet.

## Required Common Arguments — Passed By CLI or Specified in `snfoundry.toml`

* [`account`](./common.md#--account--a-account_name)

## `--class-hash, -g <CLASS_HASH>`
Required.

Class hash of contract to deploy.

## `--url, -u <RPC_URL>`
Optional.

Starknet RPC node url address.

Overrides url from `snfoundry.toml`.

## `--network <NETWORK>`
Optional.

Use predefined network with public provider

Possible values: `mainnet`, `sepolia`.

## `--constructor-calldata, -c <CONSTRUCTOR_CALLDATA>`
Optional.
Conflicts with: [`--arguments`](#--arguments)

Calldata for the contract constructor.

## `--arguments`
Optional.
Conflicts with: [`--constructor-calldata`](#--constructor-calldata--c-constructor_calldata)

Constructor arguments provided as a comma-separated string of Cairo expressions.
For example: `--arguments '1, 2, MyStruct { x: 1, y: 2 }, MyEnum::Variant'`

For more information on supported expressions and syntax, see [Calldata Transformation](../../starknet/calldata-transformation.md).

## `--salt, -s <SALT>`
Optional.

Salt for the contract address.

## `--unique`
Optional.

If passed, the salt will be additionally modified with an account address.

## `--max-fee, -m <MAX_FEE>`
Optional.

Maximum fee for the `deploy` denoted in FRI. Must be greater than zero. If provided, it is not possible to use any of the following fee related flags: `--l1-gas`, `--l1-data-price`, `--l2-gas`, `--l2-gas-price`, `--l1-data-gas`, `--l1-data-gas-price`.

## `--l1-gas <L1_GAS>`
Optional.

Maximum L1 gas for the `deploy` transaction. When not used, defaults to auto-estimation.

## ` --l1-gas-price <l1_gas_price>`
Optional.

Maximum L1 gas unit price for the `deploy` transaction. When not used, defaults to auto-estimation.

## `--l2-gas <L2_GAS>`
Optional.

Maximum L2 gas for the `deploy` transaction. When not used, defaults to auto-estimation.

## `--l2-gas-price <L2_GAS_PRICE>`
Optional.

Maximum L2 gas unit price for the `deploy` transaction. When not used, defaults to auto-estimation.

## `--l1-data-gas <L1_DATA_GAS>`
Optional.

Maximum L1 data gas for the `deploy` transaction. When not used, defaults to auto-estimation.

## `--l1-data-gas-price <l1_data_gas_price>`
Optional.

Maximum L1 data gas unit price for the `deploy` transaction. When not used, defaults to auto-estimation.

## `--nonce, -n <NONCE>`
Optional.

Nonce for transaction. If not provided, nonce will be set automatically.
