# `delete`
Delete an account from `accounts-file`.

## `--name, -n <ACCOUNT_NAME>`
Required.

Account name which is going to be deleted.

## `--url, -u <RPC_URL>`
Optional.

Starknet RPC node url address.

Overrides url from `snfoundry.toml`.

## `--network <NETWORK>`
Optional.

Use predefined network with a public provider

Possible values: `mainnet`, `sepolia`, `devnet`.

Overrides network from `snfoundry.toml`.

## `--chain-id <CHAIN_ID>`
Optional.

Chain ID of the network in `accounts-file` associated with the account, e.g. `SN_SEPOLIA`. By default, the chain ID of the RPC node passed with `--url` or `--network`.

## `--network-name <NETWORK_NAME>`
Optional.

Deprecated, use `--chain-id` instead. Accepts a chain ID or one of the legacy network names `alpha-mainnet`, `alpha-sepolia` and `alpha-integration-sepolia`.

## `--yes`
Optional.

If passed, assume "yes" as answer to confirmation prompt and run non-interactively
