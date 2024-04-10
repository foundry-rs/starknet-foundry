# `ContractClass`

A struct which enables interaction with given class hash.
It can be obtained by using [declare](./declare.md), or created with an arbitrary `ClassHash`.

```rust
struct ContractClass {
    class_hash: ClassHash,
}
```

## Implemented traits

### `ContractClassTrait`

```rust
trait ContractClassTrait {
    fn precalculate_address(
        self: @ContractClass, constructor_calldata: @Array::<felt252>
    ) -> ContractAddress;

    fn deploy(
        self: @ContractClass, constructor_calldata: @Array::<felt252>
    ) -> SyscallResult<(ContractAddress, Span<felt252>)>;

    fn deploy_at(
        self: @ContractClass,
        constructor_calldata: @Array::<felt252>,
        contract_address: ContractAddress
    ) -> SyscallResult<(ContractAddress, Span<felt252>)>;

    fn new<T, +Into<T, ClassHash>>(class_hash: T) -> ContractClass;
}
```