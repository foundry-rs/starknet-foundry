# Performing Multicall

Multicall allows you to execute multiple calls in a single transaction. `sncast` comes with two interfaces:
- `sncast multicall ...` which requires passing all calls as CLI arguments
- `sncast multicall run` which uses `.toml` file

> 📝 **Note**
> Multicall executes only one transaction containing all the prepared calls. This means the fee is paid once.

## Multicall with CLI arguments

You can prepare and execute multiple calls in a single transaction using CLI arguments. To separate different calls, use `/` as a delimiter.

### Example

```shell
$ sncast multicall \
    deploy --id map_contract --class-hash 0x02a09379665a749e609b4a8459c86fe954566a6beeaddd0950e43f6c700ed321 \
    / invoke --contract-address @map_contract --function put --calldata 0x1 0x2 \
    / invoke --contract-address @map_contract --function put --calldata 0x3 0x4
```

<details>
<summary>Output:</summary>

```shell
Success: Multicall completed

Transaction Hash: 0x[..]

To see invocation details, visit:
transaction: https://sepolia.voyager.online/tx/[..]
```
</details>
<br>

Currently, `invoke` and `deploy` calls are supported. Their syntax is the same as for `sncast invoke` and `sncast deploy` commands (with additional id argument for deploy calls). For more details on the syntax of these calls, see the [invoke](../appendix/sncast/multicall/invoke.md) and [deploy](../appendix/sncast/multicall/deploy.md) command references.

> 📝 **Note**
> The example above uses `@id` syntax to reference the address of a contract deployed within the same multicall using its `id`.
> Additionally, the id can be referenced in the calldata of deploy and invoke calls 🔥.

## Multicall with file

You need to pass `--path` flag with a `.toml` file which contains desired operations that you want to execute.

You can also compose such config `.toml` file with the `sncast multicall new` command.

For a detailed CLI description, see the [multicall command reference](../appendix/sncast/multicall/multicall.md).

### Example

Let's consider the following file:

```toml
[[call]]
call_type = "deploy"
class_hash = "0x076e94149fc55e7ad9c5fe3b9af570970ae2cf51205f8452f39753e9497fe849"
inputs = []
id = "map_contract"
unique = false

[[call]]
call_type = "invoke"
contract_address = "map_contract"
function = "put"
inputs = ["0x123", 234]  # Numbers can be used directly without quotes
```

After running `sncast multicall run --path file.toml`, a declared contract will be first deployed, and then its function `put` will be invoked.

> 💡 **Info**
> Inputs can be either strings (like `"0x123"`) or numbers (like `234`).

> 📝 **Note**
> For numbers larger than 2^63 - 1 (that can't fit into `i64`), use string format (e.g., `"9223372036854775808"`) due to TOML parser limitations.

<!-- TODO: Adjust snippet and check remove ignoring output -->
<!-- { "ignored_output": true } -->
```shell
$ sncast multicall run --path multicall_example.toml
```

<details>
<summary>Output:</summary>

```shell
Success: Multicall completed

Transaction Hash: 0x[..]

To see invocation details, visit:
transaction: https://sepolia.voyager.online/tx/[..]
```
</details>
<br>

You can also generate multicall template with `multicall new` command, specifying output path.
```shell
$ sncast multicall new ./template.toml
```

<details>
<summary>Output:</summary>

```shell
Success: Multicall template created successfully

Path:    ./template.toml
Content: [[call]]

call_type = "deploy"
class_hash = ""
inputs = []
id = ""
unique = false

[[call]]
call_type = "invoke"
contract_address = ""
function = ""
inputs = []
```
</details>
<br>

> ⚠️ **Warning**
> Trying to pass any existing file as an output for `multicall new` will result in error, as the command doesn't overwrite by default.
> You can use `--overwrite` flag to allow overwriting existing files.
