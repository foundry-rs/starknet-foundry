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

Max fee for transaction. If not provided, max fee will be automatically estimated.

## `--nonce, -n <NONCE>`
Optional.

Nonce for transaction. If not provided, nonce will be set automatically.

## `--package <NAME>`
Optional.

Name of the package that should be used.

If supplied, a contract from this package will be used. Required if more than one package exists in a workspace.