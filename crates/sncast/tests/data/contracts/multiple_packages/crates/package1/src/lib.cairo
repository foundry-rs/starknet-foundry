#[starknet::contract]
mod supercomplexcode1 {
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    fn whatever(ref self: ContractState) -> felt252 {
        1
    }
}
