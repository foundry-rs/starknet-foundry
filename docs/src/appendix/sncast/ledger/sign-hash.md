# `ledger sign-hash`

Blind-sign a raw hash using the connected Ledger device.

> ⚠️ **Warning**
>
> Blind signing a raw hash could be dangerous. Only sign hashes from trusted sources. If you are sending a transaction, [use Ledger as a signer](../../../starknet/ledger.md#using-ledger-as-signer) instead.

One of `--path` or `--account-id` is required.

## `--path <HD_PATH>`

[EIP-2645 derivation path](../../../starknet/eip-2645-hd-paths.md) (e.g., `m//starknet'/sncast'/0'/1'/0`).

Conflicts with: [`--account-id`](#--account-id-account_id)

## `--account-id <ACCOUNT_ID>`

Shorthand for `--path`. The account ID is used to derive the path `m//starknet'/sncast'/0'/<account-id>'/0`.

Conflicts with: [`--path`](#--path-hd_path)

## `<HASH>`
Required (positional).

The hash to sign, as a hex string with or without `0x` prefix.

## Example

<!-- { "requires_ledger": true } -->
```shell
$ sncast ledger sign-hash \
    --path "m//starknet'/sncast'/0'/1'/0" \
    0x0111111111111111111111111111111111111111111111111111111111111111
```

<!-- { "requires_ledger": true } -->
```shell
$ sncast ledger sign-hash \
    --account-id 1 \
    0x0111111111111111111111111111111111111111111111111111111111111111
```

<details>
<summary>Output:</summary>

```shell
Hash signature:
r: 0x[..]
s: 0x[..]
```
</details>
<br>
