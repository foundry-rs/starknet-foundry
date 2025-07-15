# `sncast` common flags
These flags must be specified directly after the `sncast` command and before the subcommand name.

## `--profile, -p <PROFILE_NAME>`
Optional.

Used for both `snfoundry.toml` and `Scarb.toml` if specified.
Defaults to `default` (`snfoundry.toml`) and `dev` (`Scarb.toml`).

## `--account, -a <ACCOUNT_NAME>`
Optional.

Account name used to interact with the network, aliased in the accounts file.

Overrides account from `snfoundry.toml`.

If used with `--keystore`, should be a path to [starkli account JSON file](https://book.starkli.rs/accounts#accounts).

## `--accounts-file, -f <PATH_TO_ACCOUNTS_FILE>`
Optional.

Path to the accounts file holding accounts information. Defaults to `~/.starknet_accounts/starknet_open_zeppelin_accounts.json`.

## `--keystore, -k <PATH_TO_KEYSTORE_FILE>`
Optional.

Path to [keystore file](https://book.starkli.rs/signers#encrypted-keystores).
When specified, the --account argument must be a path to [starkli account JSON file](https://book.starkli.rs/accounts#accounts).

## `--json, -j`
Optional.

If passed, output will be displayed in json format.

## `--wait, -w`
Optional.

If passed, command will wait until transaction is accepted or rejected.

## `--wait-timeout <TIME_IN_SECONDS>`
Optional.

If `--wait` is passed, this will set the time after which `sncast` times out. Defaults to 60s.

## `--wait-retry-timeout <TIME_IN_SECONDS>`
Optional.

If `--wait` is passed, this will set the retry interval - how often `sncast` should fetch tx info from the node. Defaults to 5s.

## `--version, -v`

Prints out `sncast` version.

## `--help, -h`

Prints out help.
