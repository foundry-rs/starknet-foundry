# `declare`
Send a declare transaction of Cairo contract to Starknet.

## Required Common Arguments — Passed By CLI or Specified in `snfoundry.toml`

* [`url`](./common.md#--url--u-rpc_url)
* [`account`](./common.md#--account--a-account_name)

## `--contract-name, -c <CONTRACT_NAME>`
Required.

Name of the contract. Contract name is a part after the mod keyword in your contract file.

## `--max-fee, -m <MAX_FEE>`
Optional.

Maximum fee for the `declare` transaction in Fri or Wei depending on fee token or transaction version. When not used, defaults to auto-estimation.

## `--fee-token <FEE_TOKEN>`
Optional. Required if `--version` is not provided.

Token used for fee payment. Possible values: ETH, STRK.

## `--max-gas <MAX_GAS>`
Optional.

Maximum gas for the `declare` transaction. When not used, defaults to auto-estimation. (Only for STRK fee payment)

## ` --max-gas-unit-price <MAX_GAS_UNIT_PRICE>`
Optional.

Maximum gas unit price for the `declare` transaction paid in Fri. When not used, defaults to auto-estimation. (Only for STRK fee payment)

## `--version, -v <VERSION>`
Optional. Required if `--fee-token` is not provided.

Version of the deployment transaction. Possible values: v2, v3.

## `--nonce, -n <NONCE>`
Optional.

Nonce for transaction. If not provided, nonce will be set automatically.

## `--package <NAME>`
Optional.

Name of the package that should be used.

If supplied, a contract from this package will be used. Required if more than one package exists in a workspace.