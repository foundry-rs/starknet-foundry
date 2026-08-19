# `get tx-trace`

Get the execution trace of a transaction.

This command is also available as `get transaction-trace`.

## `<TRANSACTION_HASH>`

Required.

Hash of the transaction.

## `--url, -u <RPC_URL>`
Optional.

Starknet RPC node url address.

Overrides url from `snfoundry.toml`.

## `--network <NETWORK>`
Optional.

Use predefined network with public provider

Possible values: `mainnet`, `sepolia`, `devnet`.

Overrides network from `snfoundry.toml`.

## `--full`
Optional.

Display full transaction trace. Without this flag, the output will omit some fields for brevity.
