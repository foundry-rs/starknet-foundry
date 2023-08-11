# `l1_handler_call`

> `fn l1_handler_call(prepared_l1_handler: PreparedL1Handler)`

Calls a `#[l1_handler]` function to mock a
[message](https://docs.starknet.io/documentation/architecture_and_concepts/L1-L2_Communication/messaging-mechanism/)
arriving from Ethereum.

```rust
struct PreparedL1Handler {
    contract_address: ContractAddress,
    selector: felt252,
    from_address: felt252,
    payload: Span::<felt252>,
}
```

where:

- `contract_address` - target contract address
- `selector` - selector of a `#[l1_handler]` function
- `from_address` - ethereum address which sends the message
- `payload` - message payload

It is important to note that the `from_address` must always be
checked as any L1 contract can call any L2 contract.

The payload contains any Cairo data structure that can be serialized with
[Serde](https://book.cairo-lang.org/appendix-03-derivable-traits.html?highlight=serde#serializing-with-serde).

For contract implementation:

```rust
// ...
#[storage]
struct Storage {
    l1_caller: felt252,
    //...
}

//...

#[l1_handler]
fn process_l1_message(ref self: ContractState, from_address: felt252, data: Span<felt252>) {
    assert(from_address == self.l1_caller.read(), 'Unauthorized l1 caller');
}
// ...
```

We can use `l1_handler_call` to call a `l1_handler`, which is not accessible using a dispatcher:

```rust
use snforge_std::l1_handler_call;

#[test]
fn test_l1_handler_call() {
    // ...
    let data: Array<felt252> = array![1, 2];

    let mut payload_buffer: Array<felt252> = ArrayTrait::new();
    data.serialize(ref payload_buffer);

    let l1_handler_prepared = PreparedL1Handler {
        contract_address,
        selector: 0x01e6b389ca484cb6fb23cbbcaa2db5581a8d970e3c135e8170c2ea5fdc2d3d8e,
        from_address: 0x123,
        payload: payload_buffer.span(),
    };

    l1_handler_call(l1_handler_prepared);
    //...
}
```
