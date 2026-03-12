# `nonce`

Get the nonce of the specified contract.

## `<CONTRACT_ADDRESS>`

Required.

Address of the contract in hex (prefixed with '0x') or decimal representation.

## `--block-id, -b <BLOCK_ID>`
Optional.

Block identifier on which nonce should be fetched.
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
