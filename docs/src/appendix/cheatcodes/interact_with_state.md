# `interact_with_state`

> ⚠️ **Warning**
>
> The feature might not work correctly if you're using Cairo version earlier than `2.11`, because closures weren’t fully supported before it.

> `pub fn interact_with_state<F, +Drop<F>, impl func: core::ops::FnOnce<F, ()>, +Drop<func::Output>>(contract_address: ContractAddress, f: F,) -> func::Output`

Allows to use `contract_state_for_testing` for a deployed contract, enabling interaction with its state in tests.

To use this cheatcode, it is necessary to take care of the following:

- The contract implementation must be visible in the test context
- Storage struct along with the variables that you want to access must be public
- If testing internal contract functions, the respective trait or specific function must be imported
- Storage related traits must be imported, such as `StoragePointerReadAccess` and `StoragePointerWriteAccess`

> 📝 **Note**
> To use `interact_with_state` with a forked contract, it is required to have the contract's implementation.

### Usage

To use this cheatcode, follow these steps:

1. Provide the contract address as the first argument
2. Define a closure that modifies the contract's state and pass it as the second argument
3. Inside the closure, define a mutable variable for the contract's state using `contract_state_for_testing`
4. Use this state variable to interact with the contract state

Here’s a minimal example that modifies a single storage variable in a contract:

```rust
interact_with_state(contract_address, || {
    let mut state = MyContract::contract_state_for_testing();
    state.balance.write(1000);
});

```

For more extensive example usage, please refer to the [testing contract internals](../../testing/testing-contract-internals.md#modifying-the-state-of-an-existing-contract) documentation.
