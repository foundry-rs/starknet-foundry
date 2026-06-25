#[starknet::interface]
trait IHelloStarknet<TContractState> {}

#[starknet::contract]
pub mod HelloStarknet {

    #[storage]
    struct Storage {
        balance: felt252,
    }

    #[abi(embed_v0)]
    impl IHelloStarknetImpl of super::IHelloStarknet<ContractState> {}
}
