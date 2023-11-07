#[starknet::contract]
mod SegmentArenaUser {
    #[storage]
    struct Storage { }
    #[external(v0)]
    fn interface_function(ref self: ContractState){
        let felt_dict: Felt252Dict<felt252> = Default::default();
    }
}
