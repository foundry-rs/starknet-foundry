# `l1_handler_execute`

> `fn execute(self: L1Handler)`

Executes a `#[l1_handler]` function to mock a
[message](https://docs.starknet.io/documentation/architecture_and_concepts/L1-L2_Communication/messaging-mechanism/)
arriving from Ethereum.

```rust
struct L1Handler {
    contract_address: ContractAddress,
    function_name: felt252,
    from_address: felt252,
    fee: u128,
    payload: Span::<felt252>,
}
```

where:

- `contract_address` - The target contract address
- `function_name` - Name of the `#[l1_handler]` function
- `from_address` - Ethereum address of the contract that sends the message
- `fee` - The fees paid on L1
- `payload` - The message payload that may contain any Cairo data structure that can be serialized with
[Serde](https://book.cairo-lang.org/appendix-03-derivable-traits.html?highlight=serde#serializing-with-serde)

It is important to note that when executing the `l1_handler`,
the `from_address` must always be checked as any L1 contract can call any L2 contract.

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

We can use `execute` method to test the execution of the `#[l1_handler]` function that is
not available using a dispatcher:

```rust
use snforge_std::L1Handler;

#[test]
fn test_l1_handler_execute() {
    // ...
    let data: Array<felt252> = array![1, 2];

    let mut payload_buffer: Array<felt252> = ArrayTrait::new();
    // Note the serialization here.
    data.serialize(ref payload_buffer);

    let mut l1_handler = L1HandlerTrait::new(
        contract_address,
        function_name: 'process_l1_message'
    );

    l1_handler.from_address = 0x123;
    l1_handler.payload = payload.span();

    l1_handler.execute();
    //...
}
```
