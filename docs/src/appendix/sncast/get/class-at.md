# `get class-at`

Get the contract class definition at an address.

## `<CONTRACT_ADDRESS>`

Required.

The address of the contract. It can be either:
- Felt in hex (prefixed with `0x`) or decimal representation.
- `@alias` defined in `[sncast.<profile>.aliases]` in `snfoundry.toml`. See [aliases](../../../starknet/aliases.md).

## `--url, -u <RPC_URL>`

Optional.

Starknet RPC node url address.

Overrides url from `snfoundry.toml`.

## `--network <NETWORK>`

Optional.

Use predefined network with public provider

Possible values: `mainnet`, `sepolia`, `devnet`.

Overrides network from `snfoundry.toml`.

## `--block-id, -b <BLOCK_ID>`

Optional.

Block identifier from which class should be fetched.
Possible values: `pre_confirmed`, `latest`, block hash (0x prefixed string), and block number (u64).
`pre_confirmed` is used as a default value.
