# `run`

Execute a single multicall transaction containing every call from passed file.

## `--path, -p <PATH>`
Required.

Path to a TOML file with call declarations.

## `--max-fee, -m <MAX_FEE>`
Optional.

Max fee for the transaction. If not provided, max fee will be automatically estimated.


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

[[call]]
call_type = "deploy"
class_hash = "0x2bb3d35dba2984b3d0cd0901b4e7de5411daff6bff5e072060bcfadbbd257b1"
inputs = ["0x123", "map_contract"]
unique = false
```
