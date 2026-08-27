# `import-starkli`

Convert a starkli account JSON file and encrypted keystore into a native sncast account.

The account metadata is stored in the V2 accounts file and the existing encrypted keystore becomes its tagged signer. Subsequent commands select the account by name and do not need the global `--keystore` flag.

The account may be deployed or undeployed. Whether it is deployed, and whether its class is a legacy (Cairo 0) class, is read from the selected network rather than from the account file's `deployment.status`.

The account's implementation class hash is taken from `deployment.class_hash` or, when the file omits it, from the network for an account already deployed at its address. A class hash from the file must match the class of a deployed account, and for an undeployed account the class must be declared on the selected network. If the file omits the class hash and the account is not deployed, the import fails.

## Account address

When the account file contains `deployment.address`, that address is imported. If the file also contains `deployment.salt`, the address must match the one computed from the salt and the account settings in the file listed below.

Account files created by older versions of `sncast account create --keystore` omit `deployment.address` until the account is deployed. For these files the address is computed from the file itself, using these fields:

| Account type | Fields used to compute the address |
|---|---|
| OpenZeppelin | `variant.public_key`, `deployment.salt`, `deployment.class_hash` |
| Ready (`argent`) | `variant.owner` (or `variant.signer`), `variant.guardian`, `deployment.salt`, `deployment.class_hash` |
| Braavos | the single Stark signer, `deployment.salt`, `deployment.context.variant` set to `braavos`, `deployment.context.base_account_class_hash` (or `mock_implementation`) |

A Braavos address is computed from its base account class rather than its implementation class, so a Braavos file without `deployment.class_hash` can still be imported when the account is already deployed at the computed address and the network provides its class hash.

Ready addresses assume the Cairo 1 account (v0.4.0 or later). If a required field is missing or malformed, the import fails without writing anything. Addresses of legacy (Cairo 0) proxy accounts, which use `variant.implementation`, are not computed. Other account files without an address, for example those of accounts other tools could not deploy, are not supported.

The native account does not retain a Ready account's guardian or a Braavos account's base account class. `sncast account deploy` deploys Ready accounts without a guardian and Braavos accounts with sncast's default base account class. An undeployed account using a guardian or another base account class must be deployed with the tool that created it; `sncast account deploy` refuses to deploy it because the resulting address would differ from the imported one.

## `--account-file <PATH>`
Required. Path to the starkli JSON account file.

## `--keystore <PATH>`
Required. Path to the existing starkli encrypted keystore.

## `--name, -n <NAME>`
Optional. Name of the native account. A name is generated when omitted.

## `--keystore-password-env <ENVIRONMENT_VARIABLE>`
Optional. Environment variable containing the password and recorded as the account's future password source.

## `--url, -u <RPC_URL>` / `--network <NETWORK>`
Select the network. The account is stored under the chain ID of this network, e.g. `SN_SEPOLIA`.

## `--add-profile <NAME>`
Optional. Add a native account profile to `snfoundry.toml`.

## `--silent`
Optional. Do not prompt to make the imported account the default.
