# `get block`

Get a block with transaction hashes.

## `<BLOCK_ID>`
Optional.

Block identifier on which the block should be fetched.
Possible values: `pre_confirmed`, `latest`, block hash (0x prefixed string), and block number (u64).
`pre_confirmed` is used as a default value.

## `--full`
Optional.

Retrieve full transactions instead of only their hashes.

## `--receipts`
Optional.

Retrieve full transactions along with their receipts. Cannot be used together with `--full`.

## `--url, -u <RPC_URL>`
Optional.

Starknet RPC node url address.

Overrides url from `snfoundry.toml`.

## `--network <NETWORK>`
Optional.

Use predefined network with public provider

Possible values: `mainnet`, `sepolia`, `devnet`.

Overrides network from `snfoundry.toml`.
