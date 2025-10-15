# `balance`
Fetch balance of the account for specified token.

## `--token, -t <TOKEN>`
Optional.
Conflicts with: [`--token-address`](#--token-address)

Name of the token to check the balance for. Possible values
- `strk` (default)
- `eth`

## `--token-address, -d <TOKEN_ADDRESS>`
Optional.
Conflicts with: [`--token`](#--token)

Token contract address to check the balance for.

## `--block-id, -b <BLOCK_ID>`
Optional.

Block identifier on which balance should be fetched
Possible values: `pending`, `latest`, block hash (0x prefixed string), and block number (u64).
`pending` is used as a default value.

## `--url, -u <RPC_URL>`
Optional.

Starknet RPC node url address.

Overrides url from `snfoundry.toml`.

## `--network <NETWORK>`
Optional.

Use predefined network with public provider

Possible values: `mainnet`, `sepolia`.
