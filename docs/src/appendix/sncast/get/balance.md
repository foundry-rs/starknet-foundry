# `get balance`
Fetch balance of the account or contract for specified token.

## `--contract-address <CONTRACT_ADDRESS>`
Optional.

Contract address to check the balance for. 
Cannot be used together with `--account` or an account from `snfoundry.toml`.
It can be either:
- Felt in hex (prefixed with `0x`) or decimal representation.
- `@alias` defined in `[sncast.<profile>.aliases]` in `snfoundry.toml`. See [aliases](../../../starknet/aliases.md).

## `--token, -t <TOKEN>`
Optional.
Conflicts with: [`--token-address`](#--token-address--d-token_address)

Name of the token to check the balance for. Possible values
- `strk` (default)
- `eth`

## `--token-address, -d <TOKEN_ADDRESS>`
Optional.
Conflicts with: [`--token`](#--token--t-token)

Token contract address to check the balance for. Token needs to be compatible with ERC-20 standard. It can be either:
- Felt in hex (prefixed with `0x`) or decimal representation.
- `@alias` defined in `[sncast.<profile>.aliases]` in `snfoundry.toml`. See [aliases](../../../starknet/aliases.md).

## `--block-id, -b <BLOCK_ID>`
Optional.

Block identifier on which balance should be fetched.
Possible values: `pre_confirmed`, `latest`, block hash (0x prefixed string), and block number (u64).
`pre_confirmed` is used as a default value.

## `--url, -u <RPC_URL>`
Optional.

Starknet RPC node url address.

Overrides url from `snfoundry.toml`.

## `--network <NETWORK>`
Optional.

Use predefined network with public provider

Possible values: `mainnet`, `sepolia`, `devnet`.

Overrides network from `snfoundry.toml`.
