# `interact_with_state`

> `pub fn interact_with_state<F, +Drop<F>, impl func: core::ops::FnOnce<F, ()>, +Drop<func::Output>>(contract_address: ContractAddress, f: F,) -> func::Output`

Allows to use `contract_state_for_testing` for a deployed contract, enabling interaction with its state in tests.

To make it possible to use this cheatcode, it is necessary to take care of the following:
- The contract must be visible in the test context
- Storage struct along with the variables that you want to access must be public
- If testing internal contract functions, the respective trait or specific function must be imported
- Storage related traits must be imported, such as `StoragePointerReadAccess` and `StoragePointerWriteAccess`

Please note that to use `interact_with_state` with a forked contract, it is required to have the contract's implementation.

For example usage, please refer to the [testing contract internals](../../testing/testing-contract-internals.md#modifying-the-state-of-an-existing-contract) documentation.

> ⚠️ **Warning**
>
> Full support for closures was introduced in Cairo `2.11`. Therefore, this cheatcode may produce unexpected results or fail in earlier versions.
