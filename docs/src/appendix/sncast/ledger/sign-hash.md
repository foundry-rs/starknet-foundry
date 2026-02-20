# `ledger sign-hash`

Blind-sign a raw hash using the connected Ledger device.

> ⚠️ **Warning**
>
> Blind signing a raw hash could be dangerous. Only sign hashes from trusted sources. If you are sending a transaction, use `--ledger-path` as a signer instead.

## `--path <HD_PATH>`
Required.

EIP-2645 derivation path (e.g., `m//starknet'/sncast'/0'/0'/0`).

## `<HASH>`
Required (positional).

The hash to sign, as a hex string with or without `0x` prefix.

## Example

<!-- { "requires_ledger": true } -->
```shell
$ sncast ledger sign-hash \
    --path "m//starknet'/sncast'/0'/0'/0" \
    0x0111111111111111111111111111111111111111111111111111111111111111
```

```shell
Signature: 0x[..]
```
