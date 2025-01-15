# `delete`
Delete an account from `accounts-file` and its associated snfoundry profile.

## `--name, -n <ACCOUNT_NAME>`
Required.

Account name which is going to be deleted.

## `--url, -u <RPC_URL>`
Optional.

Starknet RPC node url address.

Overrides url from `snfoundry.toml`.

## `--network <NETWORK>`
Optional.

Use predefined network using public provider

Possible values: `mainnet`, `sepolia`.

## `--network-name`
Optional.

Network in `accounts-file` associated with the account. By default, the network passed as `--network` of RPC node.

## `--yes`
Optional.

If passed, assume "yes" as answer to confirmation prompt and run non-interactively
