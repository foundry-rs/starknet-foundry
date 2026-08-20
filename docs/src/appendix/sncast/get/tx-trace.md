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

Display full transaction trace.

By default, the command displays a compact trace focused on the invocation flow. For each invocation, it shows the entry point selector, contract address, calldata, result, and any nested calls.

Use the `--full` flag to include the remaining trace data, such as call and entry point types, caller and class information, emitted events, L1 messages, execution resources, revert status, and the transaction state diff.
