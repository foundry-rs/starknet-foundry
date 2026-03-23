# `deploy`
Deploy a contract to Starknet.

## Required Common Arguments — Passed By CLI or Specified in `snfoundry.toml`

* [`account`](./common.md#--account--a-account_name)

## `--class-hash, -g <CLASS_HASH>`
Required if `--contract-name` is not provided.

Class hash of contract to deploy.

## `--contract-name <CONTRACT_NAME>`
Required if `--class-hash` is not provided.

Name of the contract to deploy. Can be used instead of `--class-hash`. Requires `--package` if more than one package exists in a workspace.

## `--package <NAME>`
Optional.

Name of the package that should be used.

If supplied, a contract from this package will be used. Required if more than one package exists in a workspace.

## `--url, -u <RPC_URL>`
Optional.

Starknet RPC node url address.

Overrides url from `snfoundry.toml`.

## `--network <NETWORK>`
Optional.

Use predefined network with public provider

Possible values: `mainnet`, `sepolia`, `devnet`.

Overrides network from `snfoundry.toml`.

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

Maximum fee for the `deploy` denoted in FRI. Must be greater than zero. If provided, it is not possible to use any of the following fee related flags: `--l1-gas`, `--l1-gas-price`, `--l2-gas`, `--l2-gas-price`, `--l1-data-gas`, `--l1-data-gas-price`.

## `--l1-gas <L1_GAS>`
Optional.

Maximum L1 gas for the `deploy` transaction. When not used, defaults to auto-estimation.

## `--l1-gas-price <l1_gas_price>`
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

## `--tip <TIP>`
Optional.
Conflicts with: [`--estimate-tip`](#--estimate-tip-estimate_tip)

Tip for the transaction. When not provided, defaults to 0 unless [`--estimate-tip`](#--estimate-tip-estimate_tip) is used.

## `--estimate-tip`
Optional.
Conflicts with: [`--tip`](#--tip-tip)

If passed, an estimated tip will be added to pay for the transaction. The tip is estimated based on the current network conditions and added to the transaction fee.

## `--dry-run`
Optional.

If passed, the transaction will not be sent to the network and the fee will be estimated instead. See also: [`--detailed`](#--detailed).

## `--detailed`
Optional.

If passed, the output will include detailed fee estimation results instead of just overall fee. Requires [`--dry-run`](#--dry-run) flag to be used.

## `--nonce, -n <NONCE>`
Optional.

Nonce for transaction. If not provided, nonce will be set automatically.
