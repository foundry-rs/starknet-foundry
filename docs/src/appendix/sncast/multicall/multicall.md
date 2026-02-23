# `multicall`
Provides utilities for performing multicalls on Starknet.

Multicall has the following subcommands:
* [`new`](./new.md)
* [`run`](./run.md)

When using the CLI-arguments interface, `multicall` supports the following call types:
* [`deploy`](./deploy.md)
* [`invoke`](./invoke.md)

Subsequent calls need to be separated with a `/` delimiter. For example: `sncast multicall deploy ... / invoke ... / deploy ...`

When using the file interface, the call types are specified in the `.toml` config file.

## `--url, -u <RPC_URL>`
Optional.

Starknet RPC node url address.

Overrides url from `snfoundry.toml`.

## `--network <NETWORK>`
Optional.

Use predefined network with public provider

Possible values: `mainnet`, `sepolia`, `devnet`.

Overrides network from `snfoundry.toml`.

## `--max-fee, -m <MAX_FEE>`
Optional.

Maximum fee for the `invoke` denoted in FRI. Must be greater than zero. If provided, it is not possible to use any of the following fee related flags: `--l1-gas`, `--l1-data-price`, `--l2-gas`, `--l2-gas-price`, `--l1-data-gas`, `--l1-data-gas-price`.

## `--l1-gas <L1_GAS>`
Optional.

Maximum L1 gas for the `invoke` transaction. When not used, defaults to auto-estimation.

## ` --l1-gas-price <l1_gas_price>`
Optional.

Maximum L1 gas unit price for the `invoke` transaction. When not used, defaults to auto-estimation.

## `--l2-gas <L2_GAS>`
Optional.

Maximum L2 gas for the `invoke` transaction. When not used, defaults to auto-estimation.

## `--l2-gas-price <L2_GAS_PRICE>`
Optional.

Maximum L2 gas unit price for the `invoke` transaction. When not used, defaults to auto-estimation.

## `--l1-data-gas <L1_DATA_GAS>`
Optional.

Maximum L1 data gas for the `invoke` transaction. When not used, defaults to auto-estimation.

## `--l1-data-gas-price <l1_data_gas_price>`
Optional.

Maximum L1 data gas unit price for the `invoke` transaction. When not used, defaults to auto-estimation.

## `--tip <TIP>`
Optional.
Conflicts with: [`--estimate-tip`](#--estimate-tip-estimate_tip)

Tip for the transaction. When not provided, defaults to 0 unless [`--estimate-tip`](#--estimate-tip-estimate_tip) is used.

## `--estimate-tip <ESTIMATE_TIP>`
Optional.
Conflicts with: [`--tip`](#--tip-tip)

If passed, an estimated tip will be added to pay for the transaction. The tip is estimated based on the current network conditions and added to the transaction fee.

## `--nonce, -n <NONCE>`
Optional.

Nonce for transaction. If not provided, nonce will be set automatically.
