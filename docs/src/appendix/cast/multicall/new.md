# `new`

Generates an empty template for the multicall `.toml` file that may be later used with the `run` subcommand. It either outputs it to a new file or to the standard output.

## `--output-path, -p <PATH>`
Optional.

Specifies a file path where the template should be saved.
If omitted, the template contents will be printed out to the stdout.

## `--overwrite, -o <OVERWRITE>`
Optional.

If the file specified by `--output-path` already exists, this parameter overwrites it.
