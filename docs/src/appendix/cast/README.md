# `cast` commands

## Common flags

### `--profile, -p <PROFILE_NAME>`
Optional.

Profile name in `Scarb.toml` config file.

### `--path-to-scarb-toml, -s <PATH>`
Optional.

Path to `Scarb.toml` file.

If supplied, cast will not look for `Scarb.toml` file in current (or parent) directory, but will use this path instead.

### `--url, -u <RPC_URL>`
Optional.

Starknet RPC node url address.

Overrides url from `Scarb.toml`.

### `--network, -n <NETWORK_NAME>`
Optional.

Starknet network name, one of `testnet`, `testnet2`, `mainnet`.

Overrides network from `Scarb.toml`.

### `--account, -a <ACCOUNT_NAME`
Optional.

Account name used to interact with the network, aliased in open zeppelin accounts file.

Overrides account from `Scarb.toml`.

### `--accounts-file, -f <PATH_TO_ACCOUNTS_FILE>`
Optional.

Path to the open zeppelin accounts file holding accounts info. Defaults to `~/.starknet_accounts/starknet_open_zeppelin_accounts.json`.

### `--int-format, -i`
Optional.

If passed, values will be displayed as decimals instead of hexes.

### `--json, -j`
Optional.

If passed, output will be displayed in json format.

### `--version, -v`

Prints out `cast` version.

### `--help, -h`

Prints out help.


## `declare`
Send a declare transaction of Cairo contract to Starknet.

### `--contract-name, -c <CONTRACT_NAME>`
Required.

Name of the contract. Contract name is a part after the mod keyword in your contract file.

### `--max-fee, -m <MAX_FEE>`
Optional.

Max fee for transaction. If not provided, max fee will be automatically estimated.
