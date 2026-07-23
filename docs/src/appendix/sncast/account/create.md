# `create`
Prepare all prerequisites for account deployment.

Account information will be saved to the file specified by `--accounts-file` argument,
which is `~/.starknet_accounts/starknet_open_zeppelin_accounts.json` by default.

## `--name, -n <ACCOUNT_NAME>`
Optional.

Account name under which account information is going to be saved.

If `--name` is not provided, it will be generated automatically.

## `--url, -u <RPC_URL>`
Optional.

Starknet RPC node url address.

Overrides url from `snfoundry.toml`.

## `--network <NETWORK>`
Optional.

Use predefined network with a public provider

Possible values: `mainnet`, `sepolia`, `devnet`.

Overrides network from `snfoundry.toml`.

## `--type, -t <ACCOUNT_TYPE>`
Optional. Required if `--class-hash` is passed.

Type of the account. Possible values: `oz`, `ready`, `braavos`. Defaults to oz.

Versions of the account contracts:

| Account Contract | Version | Class Hash                                                                                                                                                            |
| ---------------- | ------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `oz`             | v1.0.0  | [0x05b4b537eaa2399e3aa99c4e2e0208ebd6c71bc1467938cd52c798c601e43564](https://voyager.online/class/0x05b4b537eaa2399e3aa99c4e2e0208ebd6c71bc1467938cd52c798c601e43564) |
| `ready`          | v0.4.0  | [0x036078334509b514626504edc9fb252328d1a240e4e948bef8d0c08dff45927f](https://voyager.online/class/0x036078334509b514626504edc9fb252328d1a240e4e948bef8d0c08dff45927f) |
| `braavos`        | v1.2.0  | [0x03957f9f5a1cbfe918cedc2015c85200ca51a5f7506ecb6de98a5207b759bf8a](https://voyager.online/class/0x03957f9f5a1cbfe918cedc2015c85200ca51a5f7506ecb6de98a5207b759bf8a) |

## `--salt, -s <SALT>`
Optional.

Salt for the account address. If omitted random one will be generated.

## `--add-profile <NAME>`
Optional.

If passed, a profile with corresponding name will be added to the local snfoundry.toml.

## `--class-hash, -c`
Optional.

Class hash of a custom openzeppelin account contract declared to the network. It can be either:
- Felt in hex (prefixed with `0x`) or decimal representation.
- `@alias` defined in `[sncast.<profile>.aliases]` in `snfoundry.toml`. See [aliases](../../../starknet/aliases.md).

## `--private-key <PRIVATE_KEY>`
Optional. If neither `--private-key` nor `--private-key-file` is passed, a random private key will be generated.

Account private key. It must be a valid STARK curve secret scalar, i.e. a non-zero value smaller than the curve order `0x800000000000010ffffffffffffffffb781126dcae7b2321e66a241adc64d2f`.

> 💡 **Note**
> Passing the key via `--private-key` exposes it in your shell history and in the process list. Prefer `--private-key-file` for keys you want to keep secret.

Conflicts with: [`--ledger-path`](#--ledger-path-hd_path), [`--ledger-account-id`](#--ledger-account-id-account_id)

## `--private-key-file <PRIVATE_KEY_FILE_PATH>`
Optional. If neither `--private-key` nor `--private-key-file` is passed, a random private key will be generated.

Path to the file holding account private key. The key must satisfy the same constraints as [`--private-key`](#--private-key-private_key).

Conflicts with: [`--private-key`](#--private-key-private_key), [`--ledger-path`](#--ledger-path-hd_path), [`--ledger-account-id`](#--ledger-account-id-account_id)

## `--ledger-path <HD_PATH>`
Optional.

[EIP-2645 derivation path](../../../starknet/eip-2645-hd-paths.md) of the Ledger key that will control this account (e.g., `m//starknet'/sncast'/0'/1'/0`).

When provided, the public key is read from the Ledger device.

Conflicts with: [`--private-key`](#--private-key-private_key), [`--private-key-file`](#--private-key-file-private_key_file_path), [`--ledger-account-id`](#--ledger-account-id-account_id)

See [Ledger Hardware Wallet](../../../starknet/ledger.md) for details.

## `--ledger-account-id <ACCOUNT_ID>`
Optional.

Shorthand for `--ledger-path`. The account ID is used to derive the path `m//starknet'/sncast'/0'/<account-id>'/0`.

Conflicts with: [`--ledger-path`](#--ledger-path-hd_path), [`--private-key`](#--private-key-private_key), [`--private-key-file`](#--private-key-file-private_key_file_path)

See [Ledger Hardware Wallet](../../../starknet/ledger.md) for details.
