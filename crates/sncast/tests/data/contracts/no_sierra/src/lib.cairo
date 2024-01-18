#[starknet::interface]
trait Iminimal_contract<TContractState> {
    fn empty(ref self: TContractState);
}

#[starknet::contract]
mod minimal_contract {
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl minimal_contractImpl of super::Iminimal_contract<ContractState> {
        fn empty(ref self: ContractState) {}
    }
}
