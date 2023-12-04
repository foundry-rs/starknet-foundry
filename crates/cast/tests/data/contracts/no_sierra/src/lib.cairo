#[starknet::contract]
mod minimal_contract {
    #[storage]
    struct Storage {}
    #[abi(embed_v0)]
    fn empty(ref self: ContractState) {}
}
