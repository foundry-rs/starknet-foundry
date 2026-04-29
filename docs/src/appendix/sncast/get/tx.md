# `get tx`

Get the details of a transaction. 

This command is also available as `get transaction`.

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

## `--with-proof-facts`
Optional.

If passed, includes proof facts in the transaction response (when supported by the connected RPC node).
