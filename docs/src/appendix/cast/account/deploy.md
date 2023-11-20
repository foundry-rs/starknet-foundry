# `deploy`
Deploy previously created account to Starknet.

## Required Common Arguments - Passed by CLI or Specified in Scarb.toml

* [`url`](../common.md#--url--u-rpc_url)

## `--name, -n <ACCOUNT_NAME>`
Required.

Name of the (previously created) account to be deployed.

## `--max-fee, -m <MAX_FEE>`
Required.

Max fee for deploy account transaction.

## `--class-hash, -c`
Optional.

Class hash of a custom openzeppelin account contract declared to the network.
