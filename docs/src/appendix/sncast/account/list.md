# `list`
List all available accounts.

Account information will be retrieved from the file specified in user's environment.
The output format is dependent on user's configuration, either provided via CLI or specified in `snfoundry.toml`.
Hides user's private keys by default.

> ⚠️ **Warning**
> This command outputs cryptographic information about accounts, e.g. user's private key.
> Use it responsibly to not cause any vulnerabilities to your environment and confidential data.

## Required Common Arguments — Passed By CLI or Specified in `snfoundry.toml`

* [`accounts-file`](../common.md#--accounts-file--f-path_to_accounts_file)

## Optional Common Arguments — Passed By CLI or Specified in `snfoundry.toml`

* [`json`]()

## `--display-private-keys`, `-p`
Optional.

If passed, show private keys along with the rest of the account information.

