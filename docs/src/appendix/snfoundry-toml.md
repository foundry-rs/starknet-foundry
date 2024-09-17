# The Manifest Format

The `snfoundry.toml` file, present in each package, is called its manifest. 
This file allows you to specify sncast settings. It is written in the [TOML](https://toml.io/) format. 
It contains metadata needed to configure sncast settings and should be placed in the root of your project. 
If `snfoundry.toml` is not found in the root directory, sncast will look in all parent directories. 
If it is not found, default values will be used.

Schema of the `snfoundry.toml` file:

## `[sncast.<profile-name>]`


```toml
[sncast.myprofile]
url = "http://127.0.0.1:5055/rpc"
account = "user1"
accounts_file = "../account-file"
wait_params = { timeout = 300, retry_interval = 5 }
```

There is no required fields in the profile section, it depends more on the arguments that the user wants to have 
provided in advance

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

The `account` field specifies the account to be used for transactions. This should be a valid account name listed in `accounts_file`.

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

```toml
[sncast.myprofile]
wait_params = { timeout = 300, retry-interval = 5 }
```

### `block_explorer`

The `block_explorer` field specifies the block explorer service used to display links to transaction details.

| Value     | URL                                    |
|-----------|----------------------------------------|
| StarkScan | `https://starkscan.co/search`          |
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