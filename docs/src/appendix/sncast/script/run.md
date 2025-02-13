# `run`
Compile and run a cairo deployment script.

## Required Common Arguments — Passed By CLI or Specified in `snfoundry.toml`

* [`account`](../common.md#--account--a-account_name)

## `<MODULE_NAME>`
Required.

Script module name that contains the 'main' function that will be executed.

## `--url, -u <RPC_URL>`
Optional.

Starknet RPC node url address.

Overrides url from `snfoundry.toml`.

## `--network <NETWORK>`
Optional.

Use predefined network with public provider

Possible values: `mainnet`, `sepolia`.

## `--package <NAME>`
Optional.

Name of the package that should be used.

If supplied, a script from this package will be used. Required if more than one package exists in a workspace.

## `--no-state-file`
Optional.

Do not read/write state from/to the state file.

If set, a script will not read the state from the state file, and will not write a state to it. 
