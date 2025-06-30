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

Possible values: `mainnet`, `sepolia`.

## `--type, -t <ACCOUNT_TYPE>`
Optional. Required if `--class-hash` is passed.

Type of the account. Possible values: `oz`, `ready` (formerly `argent`), `braavos`. Defaults to oz.

Versions of the account contracts:

| Account Contract | Version | Class Hash                                                                                                                                                          |
|------------------|---------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `oz`             | v1.0.0  | [0x05b4b537eaa2399e3aa99c4e2e0208ebd6c71bc1467938cd52c798c601e43564](https://starkscan.co/class/0x05b4b537eaa2399e3aa99c4e2e0208ebd6c71bc1467938cd52c798c601e43564) |
| `ready` (formerly `argent`)        | v0.4.0  | [0x036078334509b514626504edc9fb252328d1a240e4e948bef8d0c08dff45927f](https://starkscan.co/class/0x036078334509b514626504edc9fb252328d1a240e4e948bef8d0c08dff45927f) |
| `braavos`        | v1.2.0  | [0x03957f9f5a1cbfe918cedc2015c85200ca51a5f7506ecb6de98a5207b759bf8a](https://starkscan.co/class/0x03957f9f5a1cbfe918cedc2015c85200ca51a5f7506ecb6de98a5207b759bf8a) |

## `--salt, -s <SALT>`
Optional.

Salt for the account address. If omitted random one will be generated.

## `--add-profile <NAME>`
Optional.

If passed, a profile with corresponding name will be added to the local snfoundry.toml.

## `--class-hash, -c`
Optional.

Class hash of a custom openzeppelin account contract declared to the network.
