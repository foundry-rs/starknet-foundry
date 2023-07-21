# `multicall`

Multicall has the following subcommands:
- `run`
- `new`

## `run`

Execute multiple deploy (via UDC) or invoke calls ensuring atomicity.

### `--path, -p <PATH>`
Required.

Path to a TOML file with call declarations.

File example:

```toml
[[call]]
call_type = "deploy"
class_hash = "0x076e94149fc55e7ad9c5fe3b9af570970ae2cf51205f8452f39753e9497fe849"
inputs = []
id = "map_contract"
unique = false

[[call]]
call_type = "invoke"
contract_address = "0x38b7b9507ccf73d79cb42c2cc4e58cf3af1248f342112879bfdf5aa4f606cc9"
function = "put"
inputs = ["0x123", "234"]

[[call]]
call_type = "invoke"
contract_address = "map_contract"
function = "put"
inputs = ["0x123", "234"]
```

## `new`

Generates an empty template for the multicall `.toml` file that may be later use with the `run` subcommand. It either outputs it to a new file or to the standard output.

### `--output_path, -p <PATH>`
Optional.

When provided, it specifies where the template should be saved, it has to be a file path.
If omitted, the template contents are going to be printed out to the stdout.

### `--overwrite, -o <OVERWRITE>`
Optional.

If the file specified by `--output-path` already exists, this parameter overwrites it.
