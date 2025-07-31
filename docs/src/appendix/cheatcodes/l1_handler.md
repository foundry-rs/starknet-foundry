# `l1_handler`

> `fn new(target: ContractAddress, selector: felt252) -> L1Handler`

Returns a structure referring to an L1 handler function.

> `fn execute(self: L1Handler) -> SyscallResult<()>`

Mocks an L1 -> L2 message from Ethereum handled by the given L1 handler function.

## Example

Let's consider a very simple contract, which receives an L1 message with an array of numbers and prints them:

```rust
{{#include ../../../listings/cheatcodes_reference/src/l1_handler_example.cairo}}
```

Test code:

```rust
{{#include ../../../listings/cheatcodes_reference/tests/test_l1_handler.cairo}}
```

<!-- { "package_name": "cheatcodes_reference" } -->
Let's run the test:
```shell
$ snforge test test_l1_handler
```

<details>
<summary>Output:</summary>

```shell
Collected 1 test(s) from cheatcodes_reference package
Running 1 test(s) from tests/
L1 message received from: 0x123
Numbers: [1, 2, 3]
[PASS] cheatcodes_reference_integrationtest::test_l1_handler::test_l1_handler ([..])
Running 0 test(s) from src/
Tests: 1 passed, 0 failed, 0 ignored, [..] filtered out
```
</details>
<br>
