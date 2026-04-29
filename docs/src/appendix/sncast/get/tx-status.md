# `get tx-status`

Get the status of a transaction. 

This command is also available as `get transaction-status`.

## `<TRANSACTION_HASH>`

Required.

Hash of the transaction

## `--url, -u <RPC_URL>`
Optional.

Starknet RPC node url address.

Overrides url from `snfoundry.toml`.

## `--network <NETWORK>`
Optional.

Use predefined network with public provider

Possible values: `mainnet`, `sepolia`, `devnet`.

Overrides network from `snfoundry.toml`.
