# `cast` Commands

### `--profile, -p <PROFILE_NAME>`
Optional.

Profile name in Scarb.toml config file.

Examples:

Profile named `profile1`
```toml
(...)
[tool.protostar.profile1]
account = "user"
(...)
```

No profile, but Scarb.toml exists and is configured for project:
```toml
(...)
[tool.protostar]
network = "testnet"
(...)
```

### `--path-to-scarb-toml, -s <PATH>`
Optional.

Path to Scarb.toml file.

If supplied, cast will not look for Scarb.toml file in current (or parent) directory, but will use this path instead.

### `--url, -u <RPC_URL>`
Optional.

Starknet RPC node url address.

Overrides rpc_url from Scarb.toml.

### `--network, -n <NETWORK_NAME>`
Optional.

Starknet network name, one of `testnet`, `testnet2`, `mainnet`.

Overrides network from Scarb.toml.

### `--account, -a <ACCOUNT_NAME`
Optional.

Account name used to interact with the network, aliased in open zeppelin accounts file.

Overrides account from Scarb.toml.

### `--accounts-file, -f <PATH_TO_ACCOUNTS_FILE>`
Optional.

Path to the open zeppelin accounts file holding accounts info. Defaults to `~/.starknet_accounts/starknet_open_zeppelin_accounts.json`.

### `--int-format, -i`
Optional.

If passed, values will be displayed as integers instead of hexes.

### `--json, -j`
Optional.

If passed, output will be displayed in json format.

### `--version, -v`

Prints out cast version.

### `--help, -h`

Prints out help.


## `declare`
Send a declare transaction of cairo contract to Starknet.

### `--contract-name, -c <CONTRACT_NAME>`
Required.

Name of the contract.

### `--max-fee, -m <MAX_FEE>`
Optional.

Max fee for transaction. If not provided, max fee will be automatically estimated.


## `call`
Call a smart contract on Starknet with given parameters.

### `--contract-address, -a <CONTRACT_ADDRESS>`
Required.

The address of the contract being called in hex (prefixed with '0x') or decimal representation.

### `--function-name, -f <FUNCTION_NAME>`
Required.

The name of the function being called.

#### `--calldata, -c <CALLDATA>`
Optional.

Inputs to the function, represented by a list of space-delimited values `0x1 2 0x3`.
Calldata arguments may be either hex or decimal felts.

### `--block-id, -b <BLOCK_ID>`
Optional.

Block identifier on which call should be performed.
Possible values: pending, latest, block hash (0x prefixed string) and block number (u64).
`pending` is used as a default value.
