# `ledger get-public-key`

Read the public key at a given [EIP-2645 derivation path](../../../starknet/eip-2645-hd-paths.md) from the connected Ledger device.

## `--path <HD_PATH>`
Required.

EIP-2645 derivation path (e.g., `m//starknet'/sncast'/0'/0'/0`).

## `--no-display`
Optional.

If passed, the public key will **not** be shown on the Ledger screen for confirmation. By default, confirmation is requested on the device.

## Example

<!-- { "requires_ledger": true } -->
```shell
$ sncast ledger get-public-key --path "m//starknet'/sncast'/0'/0'/0"
```

```shell
Public Key: 0x[..]
```
