# `import-starkli`

Convert a starkli account JSON file and encrypted keystore into a native sncast account.

The account metadata is stored in the V2 accounts file and the existing encrypted keystore becomes its tagged signer. Subsequent commands select the account by name and do not need the global `--keystore` flag.

## `--account-file <PATH>`
Required. Path to the starkli JSON account file.

## `--keystore <PATH>`
Required. Path to the existing starkli encrypted keystore.

## `--name, -n <NAME>`
Optional. Name of the native account. A name is generated when omitted.

## `--keystore-password-env <ENVIRONMENT_VARIABLE>`
Optional. Environment variable containing the password and recorded as the account's future password source.

## `--url, -u <RPC_URL>` / `--network <NETWORK>`
Select the network under which the account is stored.

## `--add-profile <NAME>`
Optional. Add a native account profile to `snfoundry.toml`.

## `--silent`
Optional. Do not prompt to make the imported account the default.
