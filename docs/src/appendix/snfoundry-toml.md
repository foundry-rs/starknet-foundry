# The Manifest Format

The `snfoundry.toml` contains the project's manifest and allows specifying sncast settings.
It contains metadata needed to configure sncast settings and should be placed in the root of your project. 
If `snfoundry.toml` is not found in the root directory, sncast will look in all parent directories. 
If it is not found, default values will be used.

Schema of the `snfoundry.toml` file:

## `[sncast.<profile-name>]`


```toml
[sncast.myprofile]
url = "http://127.0.0.1:5055/rpc"
account = "user1"
```

All fields are optional and do not have to be provided. In case a field is not defined in a manifest file, it must be provided in CLI when executing a relevant `sncast` command.

### `url`

The `url` field specifies the address of RPC provider.

```toml
[sncast.myprofile]
url = "http://example.com"
```

### `accounts_file`

The `accounts_file` field specifies the path to a file containing account information. 
If not provided, the default path is `~/.starknet_accounts/starknet_open_zeppelin_accounts.json`.

```toml
[sncast.myprofile]
accounts_file = "path/to/accounts.json"
```

### `account`

The `account` field specifies which account from the `accounts_file` to use for transactions.

```toml
[sncast.myprofile]
account = "user-dev"
```

### `keystore`

The `keystore` field specifies the path to the keystore file.

```toml
[sncast.myprofile]
keystore = "path/to/keystore"
```

### `wait_params`

The `wait_params` field specifies the parameters for waiting during transactions. Default values are `timeout = 300` and `retry_interval = 5`.
This is used while waiting for transaction. Txs will be fetched every 5s with timeout of 300s - so 60 attempts.

```toml
[sncast.myprofile]
wait_params = { timeout = 300, retry-interval = 5 }
```

### `block_explorer`

The `block_explorer` field specifies the block explorer service used to display links to transaction details. 
The `Sepolia Testnet` is supported only by `Voyager` and `StarkScan` block explorers.

| Value     | URL                                    |
|-----------|----------------------------------------|
| StarkScan | `https://starkscan.co`          |
| Voyager   | `https://voyager.online`               |
| ViewBlock | `https://viewblock.io/starknet`        |
| OkLink    | `https://www.oklink.com/starknet`      |
| NftScan   | `https://starknet.nftscan.com`         |

```toml
[sncast.myprofile]
block_explorer = "StarkScan"
```

## Example file structure

```toml
# [sncast.main]
# [sncast.myprofile1]
# url = "http://127.0.0.1:5055/rpc"
# account = "mainuser"
# accounts_file = "../account-file"
# keystore = "~/keystore"
# wait_params = {{ timeout = 500, retry_interval = 10 }}
# block_explorer = "StarkScan"

# [sncast.dev]
# url = "http://127.0.0.1:5056/rpc"
# account = "devuser"
```