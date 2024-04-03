# `run`
Compile and run a cairo deployment script.

## Required Common Arguments — Passed By CLI or Specified in `snfoundry.toml`

* [`url`](../common.md#--url--u-rpc_url)
* [`account`](../common.md#--account--a-account_name)

## `<MODULE_NAME>`
Required.

Script module name that contains the 'main' function that will be executed.

## `--package <NAME>`
Optional.

Name of the package that should be used.

If supplied, a script from this package will be used. Required if more than one package exists in a workspace.

## `--no-state-file`
Optional.

Do not read/write state from/to the state file.

If set, a script will not read the state from the state file, and will not write a state to it. 
