# `import`
Import an account to accounts file.

Account information will be saved to the file specified by `--accounts-file` argument,
which is `~/.starknet_accounts/starknet_open_zeppelin_accounts.json` by default.

## `--name, -n <NAME>`
Optional.

Name of the account to be imported.

## `--address, -a <ADDRESS>`
Required.

Address of the account.

## `--type, -t <ACCOUNT_TYPE>`
Required.

<!-- TODO(#3556): Remove `argent` option once we drop Argent account type. -->
Type of the account. Possible values: `oz`, `argent`, `ready`, `braavos`.

<!-- TODO(#3556): Remove warning once we drop Argent account type. -->
> ⚠️ **Warning**
>
> Argent has rebranded as Ready. The `argent` option is deprecated, please use `ready` instead.

## `--url, -u <RPC_URL>`
Optional.

Starknet RPC node url address.

Overrides url from `snfoundry.toml`.

## `--network <NETWORK>`
Optional.

Use predefined network with public provider

Possible values: `mainnet`, `sepolia`, `devnet`.

Overrides network from `snfoundry.toml`.

## `--class-hash, -c <CLASS_HASH>`
Optional.

Class hash of the account.

## `--private-key <PRIVATE_KEY>`
Optional.

Account private key.

## `--ledger-path <HD_PATH>`
Optional.

[EIP-2645 derivation path](../../../starknet/eip-2645-hd-paths.md) of the Ledger key that controls this account (e.g., `m//starknet'/sncast'/0'/1'/0`).

When provided, the public key is read from the Ledger device instead of from `--private-key`.

Conflicts with: [`--private-key`](#--private-key-private_key), [`--private-key-file`](#--private-key-file-private_key_file_path), [`--ledger-account-id`](#--ledger-account-id-account_id)

See [Ledger Hardware Wallet](../../../starknet/ledger.md) for details.

## `--ledger-account-id <ACCOUNT_ID>`
Optional.

Shorthand for `--ledger-path`. The account ID is used to derive the path `m//starknet'/sncast'/0'/<account-id>'/0`.

Conflicts with: [`--ledger-path`](#--ledger-path-hd_path), [`--private-key`](#--private-key-private_key), [`--private-key-file`](#--private-key-file-private_key_file_path)

See [Ledger Hardware Wallet](../../../starknet/ledger.md) for details.

## `--private-key-file <PRIVATE_KEY_FILE_PATH>`
Optional. If neither `--private-key` nor `--private-key-file` is passed, the user will be prompted to enter the account private key.

Path to the file holding account private key.

## `--salt, -s <SALT>`
Optional.

Salt for the account address.

## `--add-profile <NAME>`
Optional.

If passed, a profile with corresponding name will be added to the local snfoundry.toml.

## `--silent`
Optional.

If passed, the command will not trigger an interactive prompt to add an account as a default
