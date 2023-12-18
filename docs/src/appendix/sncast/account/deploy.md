# `deploy`
Deploy previously created account to Starknet.

## Required Common Arguments — Passed By CLI or Specified in `Scarb.toml`

* [`url`](../common.md#--url--u-rpc_url)

## `--name, -n <ACCOUNT_NAME>`
Required.

Name of the (previously created) account to be deployed.

## `--max-fee, -m <MAX_FEE>`
Optional.

Maximum fee for the `deploy_account` transaction. When not used, defaults to auto-estimation.

## `--class-hash, -c`
Optional.

Class hash of a custom OpenZeppelin account contract declared to the network.
