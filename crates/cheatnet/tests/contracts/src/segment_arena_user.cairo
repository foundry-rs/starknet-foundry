#[starknet::contract]
mod SegmentArenaUser {
    use core::dict::Felt252Dict;

    #[storage]
    struct Storage {}

    #[external(v0)]
    fn interface_function(ref self: ContractState) {
        let _felt_dict: Felt252Dict<felt252> = Default::default();
    }
}
