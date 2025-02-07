# `invoke`
Send an invoke transaction to Starknet.

## Required Common Arguments — Passed By CLI or Specified in `snfoundry.toml`

* [`account`](./common.md#--account--a-account_name)

## `--contract-address, -a <CONTRACT_ADDRESS>`
Required.

The address of the contract being called in hex (prefixed with '0x') or decimal representation.

## `--function, -f <FUNCTION_NAME>`
Required.

The name of the function to call.

## `--calldata, -c <CALLDATA>`
Optional.

Inputs to the function, represented by a list of space-delimited values `0x1 2 0x3`.
Calldata arguments may be either 0x hex or decimal felts.

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

## `--max-gas <MAX_GAS>`
Optional.

Maximum gas for the `invoke` transaction. When not used, defaults to auto-estimation. Must be greater than zero. (Only for STRK fee payment)

## ` --max-gas-unit-price <MAX_GAS_UNIT_PRICE>`
Optional.

Maximum gas unit price for the `invoke` transaction paid in Fri. When not used, defaults to auto-estimation. Must be greater than zero. (Only for STRK fee payment)

## `--nonce, -n <NONCE>`
Optional.

Nonce for transaction. If not provided, nonce will be set automatically.
