# `deploy`
Deploy previously created account to Starknet.

## Required Common Arguments — Passed By CLI or Specified in `snfoundry.toml`

* [`url`](../common.md#--url--u-rpc_url)

## `--name, -n <ACCOUNT_NAME>`
Required.

Name of the (previously created) account to be deployed.

## `--max-fee, -m <MAX_FEE>`
Optional.

Maximum fee for the `deploy_account` transaction. When not used, defaults to auto-estimation.

## `--fee-token <FEE_TOKEN>`
Optional. Required if `--version` is not provided.

Token used for fee payment. Possible values: ETH, STRK.

## `--max-gas <MAX_GAS>`
Optional.

Maximum gas for the `deploy_account` transaction. When not used, defaults to auto-estimation. (Only for STRK fee payment)

## ` --max-gas-unit-price <MAX_GAS_UNIT_PRICE>`
Optional.

Maximum gas unit price for the `deploy_account` transaction paid in STRK. When not used, defaults to auto-estimation. (Only for STRK fee payment)

## `--version, -v <VERSION>`
Optional. Required if `--fee-token` is not provided.

Version of the account deployment transaction. Possible values: v1, v3.