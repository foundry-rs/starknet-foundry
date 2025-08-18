# `l1_handler`

> `fn new(target: ContractAddress, selector: felt252) -> L1Handler`

Returns an `L1Handler` structure which can be used to mock sending messages from L1 for the contract to handle in function marked with `#[l1_handler]`.

```rust
#[derive(Drop, Clone)]
pub struct L1Handler {
    target: ContractAddress,
    selector: felt252,
}
```

> `fn execute(self: L1Handler) -> SyscallResult<()>`

Mocks an L1 -> L2 message from Ethereum handled by the given L1 handler function.

## Example

Let's consider a very simple contract, which receives an L1 message with an array of numbers them:

```rust
{{#include ../../../listings/cheatcodes_reference/src/l1_handler_example.cairo}}
```

Test code:

```rust
{{#include ../../../listings/cheatcodes_reference/tests/test_l1_handler.cairo}}
```

Let's run the test:

<!-- { "package_name": "cheatcodes_reference", "scarb": ">=2.11.4" } -->
```shell
$ snforge test test_l1_handler
```

<details>
<summary>Output:</summary>

```shell
Collected 1 test(s) from cheatcodes_reference package
Running 1 test(s) from tests/
[PASS] cheatcodes_reference_integrationtest::test_l1_handler::test_l1_handler ([..])
Running 0 test(s) from src/
Tests: 1 passed, 0 failed, 0 ignored, [..] filtered out
```
</details>
<br>
