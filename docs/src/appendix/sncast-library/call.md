# `call`

> `pub fn call(
    contract_address: ContractAddress, function_selector: felt252, calldata: Array::<felt252>
) -> Result<CallResult, ScriptCommandError>`

Calls a contract and returns `CallResult`.

- `contract_address` - address of the contract to call.
- `function_selector` - the selector of the function to call.
- `calldata` - inputs to the function to be called.

```rust
{{#include ../../../listings/sncast_library/scripts/call/src/lib.cairo}}
```

Structure used by the command:

```rust
#[derive(Drop, Clone, Debug)]
pub struct CallResult {
    pub data: Array::<felt252>,
}
```
