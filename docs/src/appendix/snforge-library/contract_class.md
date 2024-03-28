# `ContractClass`

A struct which enables interaction with given class hash.
It can be obtained by using [declare](./declare.md), or created with an arbitrary `ClassHash`.

```
struct ContractClass {
    class_hash: ClassHash,
}
```

## Implemented traits

### `ContractClassTrait`

```
trait ContractClassTrait {
    fn precalculate_address(
        self: @ContractClass, constructor_calldata: @Array::<felt252>
    ) -> ContractAddress;

    fn deploy(
        self: @ContractClass, constructor_calldata: @Array::<felt252>
    ) -> Result<ContractAddress, RevertedTransaction>;

    fn deploy_at(
        self: @ContractClass,
        constructor_calldata: @Array::<felt252>,
        contract_address: ContractAddress
    ) -> Result<ContractAddress, RevertedTransaction>;

    fn new<T, +Into<T, ClassHash>>(class_hash: T) -> ContractClass;
}
```