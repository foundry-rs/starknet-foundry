#[starknet::contract]
mod supercomplexcode2 {
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    fn whatever(ref self: ContractState) -> felt252 {
        2
    }
}
