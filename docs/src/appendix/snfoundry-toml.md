# The Manifest Format

The `snfoundry.toml` contains the project's manifest and allows specifying sncast settings.
You can configure sncast settings and arguments instead of providing them in the CLI along with the commands.
If `snfoundry.toml` is not found in the root directory, sncast will look for it in all parent directories. 
If it is not found, default values will be used.

## `snfoundry.toml` Contents

### `[sncast.<profile-name>]`


```toml
[sncast.myprofile]
# ...
```

All fields are optional and do not have to be provided. In case a field is not defined in a manifest file, it must be provided in CLI when executing a relevant `sncast` command.
Profiles allow you to define different sets of configurations for various environments or use cases. For more details, see the [profiles explanation](../projects/configuration.md).

The `url` field specifies the address of RPC provider.

```toml
[sncast.myprofile]
url = "http://example.com"
```

#### `accounts-file`

The `accounts-file` field specifies the path to a file containing account information. 
If not provided, the default path is `~/.starknet_accounts/starknet_open_zeppelin_accounts.json`.

```toml
[sncast.myprofile]
accounts-file = "path/to/accounts.json"
```

#### `account`

The `account` field specifies which account from the `accounts-file` to use for transactions.

```toml
[sncast.myprofile]
account = "user-dev"
```

#### `keystore`

The `keystore` field specifies the path to the keystore file.

```toml
[sncast.myprofile]
keystore = "path/to/keystore"
```

#### `wait-params`

The `wait-params` field defines the waiting parameters for transactions. By default, timeout (in seconds) is set to `300` and retry-interval (in seconds) to `5`. 
This means transactions will be checked every `5 seconds`, with a total of `60 attempts` before timing out.

```toml
[sncast.myprofile]
wait-params = { timeout = 300, retry-interval = 5 }
```

#### `show-explorer-links`
Enable printing links pointing to pages with transaction details in the chosen block explorer

```toml
[sncast.myprofile]
show-explorer-links = true
```

#### `block-explorer`

The `block-explorer` field specifies the block explorer service used to display links to transaction details. 

| Value     | URL                                    |
|-----------|----------------------------------------|
| StarkScan | `https://starkscan.co`          |
| Voyager   | `https://voyager.online`               |
| ViewBlock | `https://viewblock.io/starknet`        |
| OkLink    | `https://www.oklink.com/starknet`      |
| NftScan   | `https://starknet.nftscan.com`         |

```toml
[sncast.myprofile]
block-explorer = "StarkScan"
```

#### Complete Example of `snfoundry.toml` File

```toml
[sncast.myprofile1]
url = "http://127.0.0.1:5050/"
accounts-file = "../account-file"
account = "mainuser"
keystore = "~/keystore"
wait-params = { timeout = 500, retry-interval = 10 }
block-explorer = "StarkScan"
show-explorer-links = true

[sncast.dev]
url = "http://127.0.0.1:5056/rpc"
account = "devuser"
```