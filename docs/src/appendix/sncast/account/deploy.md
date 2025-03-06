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

## `--l1-gas <L1_GAS>`
Optional.

Maximum L1 gas for the `deploy_account` transaction. When not used, defaults to auto-estimation. (Only for STRK fee payment)

## ` --l1-gas-unit-price <L1_GAS_UNIT_PRICE>`
Optional.

Maximum L1 gas unit price for the `deploy_account` transaction paid in Fri. When not used, defaults to auto-estimation. (Only for STRK fee payment)

## `--l2-gas <L2_GAS>`
Optional.

Maximum L2 gas for the `deploy_account` transaction. When not used, defaults to auto-estimation. (Only for Wei fee payment)

## `--l2-gas-unit-price <L2_GAS_PRICE>`
Optional.

Maximum L2 gas unit price for the `deploy_account` transaction paid in Fri. When not used, defaults to auto-estimation. (Only for Wei fee payment)

## `--l1-data-gas <L1_DATA_GAS>`
Optional.

Maximum L1 data gas for the `deploy_account` transaction. When not used, defaults to auto-estimation. (Only for STRK fee payment)

## `--l1-data-gas-unit-price <L1_DATA_GAS_UNIT_PRICE>`
Optional.

Maximum L1 data gas unit price for the `deploy_account` transaction paid in Fri. When not used, defaults to auto-estimation. (Only for STRK fee payment)
