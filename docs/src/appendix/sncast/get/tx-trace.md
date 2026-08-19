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

Display every field returned in the transaction trace using the same aligned field format as
`get tx-receipt`, while preserving the nesting of the Starknet Trace API schema. This includes
transaction and per-call execution resources, function invocations, events, L1 messages, nested
calls, and the complete state diff.

Without this flag, the human-readable output is limited to the decoded call tree. JSON output always
contains the complete, unmodified transaction trace.
