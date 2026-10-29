# `add`

Create an alias and save it to `snfoundry.toml`

## `<NAME>`

Required.

Name of the alias.

## `<VALUE>`

Required.

Felt in hex (prefixed with `0x`) or decimal representation.

## `--global`

Optional.

Add the alias to the global configuration.

If this flag is not provided, local configuration is updated instead.

## `--force`

Optional.

Overwrite existing alias if already exists.

If this flag is not provided, `alias add` will throw an error
if an alias with the provided `name` exists.
