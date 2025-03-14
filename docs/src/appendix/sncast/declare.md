# `declare`
Send a declare transaction of Cairo contract to Starknet.

## Required Common Arguments — Passed By CLI or Specified in `snfoundry.toml`

* [`account`](./common.md#--account--a-account_name)

## `--contract-name, -c <CONTRACT_NAME>`
Required.

Name of the contract. Contract name is a part after the mod keyword in your contract file.

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

Maximum fee for the `declare` transaction depending on fee token or transaction version. If used, estimated overall fee can't be higher than this value. Must be greater than zero.

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


## `--nonce, -n <NONCE>`
Optional.

Nonce for transaction. If not provided, nonce will be set automatically.

## `--package <NAME>`
Optional.

Name of the package that should be used.

If supplied, a contract from this package will be used. Required if more than one package exists in a workspace.