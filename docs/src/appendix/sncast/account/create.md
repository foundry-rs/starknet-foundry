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

Type of the account. Possible values: oz, argent, braavos. Defaults to oz.

> ⚠️ **Warning**
> Creating braavos accounts is currently disabled.

Versions of the account contracts:

| Account Contract | Version | Class Hash                                                                                                                                                          |
|------------------|---------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `oz`             | v0.14.0 | [0x00e2eb8f5672af4e6a4e8a8f1b44989685e668489b0a25437733756c5a34a1d6](https://starkscan.co/class/0x00e2eb8f5672af4e6a4e8a8f1b44989685e668489b0a25437733756c5a34a1d6) |
| `argent`         | v0.3.1  | [0x036078334509b514626504edc9fb252328d1a240e4e948bef8d0c08dff45927f](https://starkscan.co/class/0x029927c8af6bccf3f6fda035981e765a7bdbf18a2dc0d630494f8758aa908e2b) |
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
