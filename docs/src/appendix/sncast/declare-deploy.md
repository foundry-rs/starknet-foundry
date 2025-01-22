# `declare-deploy`
Declare a contract and deploy it to Starknet immediately. If the contract has been already declared, behaves exactly as `deploy`.

>  ⚠️ **Warning**
> This command relies on auto-estimation and does not allow specifying max fees explicitly.
> ⚠️ **Warning**
> Only a `fee-token` can be specified. Transaction versions for both declaration and deployment are inferred from token type.

## Required Common Arguments — Passed By CLI or Specified in `snfoundry.toml`

* [`account`](./common.md#--account--a-account_name)

## `--contract-name, -c <CONTRACT_NAME>`
Required.

Name of the contract. Contract name is a part after the `mod` keyword in your contract file.

## `--fee-token <FEE_TOKEN>`
Optional. Required if `--version` is not provided.

Token used for fee payment. Possible values: ETH, STRK.

## `--package <NAME>`
Optional.

Name of the package that should be used.

If supplied, a contract from this package will be used. Required if more than one package exists in a workspace.

## `--constructor-calldata, -c <CONSTRUCTOR_CALLDATA>`
Optional.

Calldata for the contract constructor.

## `--salt, -s <SALT>`
Optional.

Salt for the contract address.

## `--unique, -u`
Optional.

If passed, the salt will be additionally modified with an account address.

## `--nonce, -n <NONCE>`
Optional.

Nonce for transaction. If not provided, nonce will be set automatically.