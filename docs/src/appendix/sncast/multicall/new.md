# `new`

Generates an empty template for the multicall `.toml` file that may be later used with the `run` subcommand. It writes it to a file provided as a required argument.

## Usage
## `multicall new <OUTPUT-PATH> [OPTIONS]`

## Arguments
`OUTPUT-PATH` - a path to a file to write the generated `.toml` to.

## `--output-path, -p <PATH>`
Optional.

Specifies a file path where the template should be saved.
If omitted, the template contents will be printed out to the stdout.

## `--overwrite, -o <OVERWRITE>`
Optional.

If the file specified by `--output-path` already exists, this parameter overwrites it.
