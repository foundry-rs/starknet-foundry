#[starknet::contract]
mod L1HandlerExample {
    #[storage]
    struct Storage {}

    #[l1_handler]
    fn handle_l1_message(ref self: ContractState, from_address: felt252, numbers: Array<felt252>) {
        assert!(from_address == 0x123, "Unexpected address");
        assert!(numbers.len() == 3, "Expected exactly 3 numbers");
    }
}
