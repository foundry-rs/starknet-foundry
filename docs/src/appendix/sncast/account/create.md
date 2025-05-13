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

<!-- TODO(#3118): Include braavos in possible types once integration is restored  -->
Type of the account. Possible values: oz, argent. Defaults to oz.

> ⚠️ **Warning**
> Creating braavos accounts is currently disabled.

Versions of the account contracts:

| Account Contract | Version | Class Hash                                                                                                                                                          |
|------------------|---------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `oz`             | v1.0.0  | [0x05b4b537eaa2399e3aa99c4e2e0208ebd6c71bc1467938cd52c798c601e43564](https://starkscan.co/class/0x05b4b537eaa2399e3aa99c4e2e0208ebd6c71bc1467938cd52c798c601e43564) |
| `argent`         | v0.3.1  | [0x029927c8af6bccf3f6fda035981e765a7bdbf18a2dc0d630494f8758aa908e2b](https://starkscan.co/class/0x029927c8af6bccf3f6fda035981e765a7bdbf18a2dc0d630494f8758aa908e2b) |
<!-- TODO(#3118): Uncomment once braavos integration is restored -->
<!-- | `braavos`        | v1.0.0  | [0x00816dd0297efc55dc1e7559020a3a825e81ef734b558f03c83325d4da7e6253](https://starkscan.co/class/0x00816dd0297efc55dc1e7559020a3a825e81ef734b558f03c83325d4da7e6253) | -->

## `--salt, -s <SALT>`
Optional.

Salt for the account address. If omitted random one will be generated.

## `--add-profile <NAME>`
Optional.

If passed, a profile with corresponding name will be added to the local snfoundry.toml.

## `--class-hash, -c`
Optional.

Class hash of a custom openzeppelin account contract declared to the network.
