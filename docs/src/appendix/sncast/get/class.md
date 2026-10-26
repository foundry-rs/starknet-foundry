# `get class`

Get the contract class definition. One of `--hash` and `--address` flag is required.

## `--hash, -g <CLASS-HASH>`

Class hash of a contract. It can be either:
- Felt in hex (prefixed with `0x`) or decimal representation.
- `@alias` defined in `[sncast.<profile>.aliases]` in `snfoundry.toml`. See [aliases](../../../starknet/aliases.md).

## `--address, -d <CONTRACT-ADDRESS>`

Address of a contract. It can be either:
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
