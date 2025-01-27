# `deploy`
Deploy previously created account to Starknet.

## `--name, -n <ACCOUNT_NAME>`
Required.

Name of the (previously created) account to be deployed.

## `--url, -u <RPC_URL>`
Optional.

Starknet RPC node url address.

Overrides url from `snfoundry.toml`.

## `--network <NETWORK>`
Optional.

Use predefined network with a public provider

Possible values: `mainnet`, `sepolia`.

## `--max-fee, -m <MAX_FEE>`
Optional.

Maximum fee for the `deploy_account` transaction in Fri or Wei depending on fee token or transaction version. When not used, defaults to auto-estimation. Must be greater than zero.

## `--fee-token <FEE_TOKEN>`
Optional. When not used, defaults to STRK.

Token used for fee payment. Possible values: ETH, STRK.

## `--max-gas <MAX_GAS>`
Optional.

Maximum gas for the `deploy_account` transaction. When not used, defaults to auto-estimation. Must be greater than zero. (Only for STRK fee payment)

## ` --max-gas-unit-price <MAX_GAS_UNIT_PRICE>`
Optional.

Maximum gas unit price for the `deploy_account` transaction paid in Fri. When not used, defaults to auto-estimation. Must be greater than zero. (Only for STRK fee payment)

## `--version, -v <VERSION>`
Optional. When not used, defaults to v3.

Version of the account deployment transaction. Possible values: v1, v3.