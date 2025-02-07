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

Calldata for the contract constructor.

## `--salt, -s <SALT>`
Optional.

Salt for the contract address.

## `--unique`
Optional.

If passed, the salt will be additionally modified with an account address.

## `--max-fee, -m <MAX_FEE>`
Optional.

Maximum fee for the `deploy` transaction in Fri or Wei depending on fee token or transaction version. When not used, defaults to auto-estimation. Must be greater than zero.

## `--max-gas <MAX_GAS>`
Optional.

Maximum gas for the `deploy` transaction. When not used, defaults to auto-estimation. Must be greater than zero. (Only for STRK fee payment)

## ` --max-gas-unit-price <MAX_GAS_UNIT_PRICE>`
Optional.

Maximum gas unit price for the `deploy` transaction paid in Fri. When not used, defaults to auto-estimation. Must be greater than zero. (Only for STRK fee payment)

## `--nonce, -n <NONCE>`
Optional.

Nonce for transaction. If not provided, nonce will be set automatically.
