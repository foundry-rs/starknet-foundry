# `ledger get-public-key`

Read the public key at a given [EIP-2645 derivation path](../../../starknet/eip-2645-hd-paths.md) from the connected Ledger device.

One of `--path` or `--account-id` is required.

## `--path <HD_PATH>`

[EIP-2645 derivation path](../../../starknet/eip-2645-hd-paths.md) (e.g., `m//starknet'/sncast'/0'/1'/0`).

Conflicts with: [`--account-id`](#--account-id-account_id)

## `--account-id <ACCOUNT_ID>`

Shorthand for `--path`. The account ID is used to derive the path `m//starknet'/sncast'/0'/<account-id>'/0`.

Conflicts with: [`--path`](#--path-hd_path)

## `--no-display`
Optional.

If passed, the public key will **not** be shown on the Ledger screen for confirmation. By default, confirmation is requested on the device.

## Example

<!-- { "requires_ledger": true } -->
```shell
$ sncast ledger get-public-key --path "m//starknet'/sncast'/0'/1'/0"
```

<!-- { "requires_ledger": true } -->
```shell
$ sncast ledger get-public-key --account-id 1
```

<details>
<summary>Output:</summary>

```shell
Public Key: 0x[..]
```
</details>
<br>
