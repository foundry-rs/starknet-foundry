# EIP-2645 Wallet Paths

Ledger derives private keys using [Hierarchical Deterministic Wallet derivation paths](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki) (HD paths). A single Ledger device can use an unlimited number of keys — any given combination of seed phrase + path always produces the same key pair.

Therefore, it's important to decide on, and **document** the HD paths used for your accounts. While it's trivial to use a key pair given an HD path, the reverse is not true — recovering which path corresponds to a given public key can be extremely difficult. Following [established patterns](#path-management-best-practices) (e.g. incrementing from zero) is strongly recommended.

## The EIP-2645 Standard

If you've used hardware wallets before, you may be familiar with paths like `m/44'/60'/0'/0/0`. This format will _not_ work with the Starknet Ledger app — it only accepts the [EIP-2645 HD path](https://github.com/ethereum/ercs/blob/master/ERCS/erc-2645.md) format:

```
m/2645'/layer'/application'/eth_address_1'/eth_address_2'/index
```

where `layer`, `application`, `eth_address_1`, `eth_address_2`, and `index` are 31-bit unsigned numbers.

## The sncast Extension

EIP-2645 paths in raw numeric form are difficult to read and write (e.g. `m/2645'/1195502025'/355113700'/0'/0'/0`). `sncast` provides two extensions to make them more user-friendly.

### Using Non-Numerical Strings

The `layer` and `application` levels are defined in EIP-2645 as hashes of names. Instead of manually computing those hashes, `sncast` lets you write the names directly:

```
m/2645'/starknet'/sncast'/0'/0'/0
```

`sncast` automatically converts the string segments into their SHA-256-derived numeric values. The hash of `"starknet"` is `1195502025`, so the two forms below are equivalent:

```
m/2645'/starknet'/sncast'/0'/0'/0
m/2645'/1195502025'/355113700'/0'/0'/0
```

> ℹ️ **Note**
>
> String-based path segments are a `sncast` extension and are not understood by other HD wallet tools. To obtain a universally accepted numeric representation, translate the path manually using the SHA-256 hash of each string segment (taking the lowest 31 bits of the last 4 bytes of the hash).

### Omitting the `2645'` Segment

Since `sncast` only works with EIP-2645 paths, the `2645'` prefix can be omitted using the `m//` shorthand:

```
m//starknet'/sncast'/0'/0'/0
```

This is equivalent to `m/2645'/starknet'/sncast'/0'/0'/0`.

## Path Validation Rules

`sncast` enforces these rules when parsing a derivation path:

- Must have exactly 6 levels after `m/`
- The first level must be `2645'` (or omitted via `m//`)
- The `layer`, `application`, `eth_address_1`, and `eth_address_2` levels must be hardened (marked with `'`)
- The `index` level must be a plain number (string names are not allowed here)

## Path Management Best Practices

The single most important rule is to **document the paths you've used**, especially if you deviate from common patterns.

`sncast` recommends the following convention:

1. Always use `m//starknet'/sncast'/` as the prefix for levels `layer` and `application`
2. Keep `eth_address_1` fixed at `0'`
3. Start `eth_address_2` at `0'` for your first account and increment it for each additional account
4. Keep `index` at `0`

Examples following this convention:

| Account | Path |
|---------|------|
| First | `m//starknet'/sncast'/0'/0'/0` |
| Second | `m//starknet'/sncast'/0'/1'/0` |
| Third | `m//starknet'/sncast'/0'/2'/0` |

> ℹ️ **Note**
>
> Some guides suggest incrementing `index` instead of `eth_address_2`. This carries a small security risk: if any single private key in the set is compromised _and_ the attacker obtains the `xpub` key at the `eth_address_2` level, they can derive all sibling keys. Incrementing `eth_address_2` avoids this.
