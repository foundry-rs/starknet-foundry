# `l1_handler_invoke`

> `fn invoke(self: L1Handler, from_address: felt252, fee: u128, payload: Span::<felt252>)`

Invokes a `#[l1_handler]` function to mock a
[message](https://docs.starknet.io/documentation/architecture_and_concepts/L1-L2_Communication/messaging-mechanism/)
arriving from Ethereum.

```rust
struct L1Handler {
    contract_address: ContractAddress,
    selector_name: felt252,
}
```

where:

- `contract_address` - target contract address
- `selector_name` - selector's name of a `#[l1_handler]` function

It is important to note that when invoking the `l1_handler`,
the `from_address` must always be checked as any L1 contract can call any L2 contract.

The payload contains any Cairo data structure that can be serialized with
[Serde](https://book.cairo-lang.org/appendix-03-derivable-traits.html?highlight=serde#serializing-with-serde).

For contract implementation:

```rust
// ...
#[storage]
struct Storage {
    l1_allowed: felt252,
    //...
}

//...

#[l1_handler]
fn process_l1_message(ref self: ContractState, from_address: felt252, data: Span<felt252>) {
    assert(from_address == self.l1_allowed.read(), 'Unauthorized l1 contract');
}
// ...
```

First, the `build` method is used to configure the `L1Handler` to the
desired contract and selector. Once built, you can invoke the handler
several times, with different parameters to test.

Invoke is used to execute `process_l1_message`, which is not accessible using a dispatcher:

```rust
use snforge_std::L1Handler;

#[test]
fn test_l1_handler_invoke() {
    // ...
    let data: Array<felt252> = array![1, 2];

    let mut payload_buffer: Array<felt252> = ArrayTrait::new();
    data.serialize(ref payload_buffer);

    let l1_handler = L1Handler {
        contract_address,
        selector_name: 'process_l1_message'
    };

    l1_handler.invoke(0x123, 1, payload.span());
    //...
}
```
