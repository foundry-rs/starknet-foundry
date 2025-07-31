#[starknet::contract]
mod L1HandlerExample {
    #[storage]
    struct Storage {}

    #[l1_handler]
    fn handle_l1_message(ref self: ContractState, from_address: felt252, numbers: Array<felt252>) {
        println!("L1 message received from: 0x{:x}", from_address);
        println!("Numbers: {:?}", numbers);
    }
}
