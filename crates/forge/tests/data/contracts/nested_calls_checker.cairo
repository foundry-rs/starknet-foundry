use starknet::ContractAddress;

#[starknet::interface]
pub trait INestedCallsChecker<TContractState> {
    fn call_other_contract(self: @TContractState, contract_address: ContractAddress);
}

#[starknet::interface]
pub trait IHelloStarknet<TContractState> {
    fn example_function(ref self: TContractState);
}

#[starknet::contract]
pub mod HelloStarknet {
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl HelloStarknetImpl of super::IHelloStarknet<ContractState> {
        fn example_function(ref self: ContractState) {
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
            core::pedersen::pedersen(1, 2);
        }
    }
}


#[starknet::contract]
pub mod NestedCallsChecker {
    use core::array::ArrayTrait;
    use starknet::ContractAddress;
    use crate::{IHelloStarknetDispatcherTrait, IHelloStarknetDispatcher};

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl INestedCallsCheckerImpl of super::INestedCallsChecker<ContractState> {
        fn call_other_contract(self: @ContractState, contract_address: ContractAddress) {
            let hello_starknet = IHelloStarknetDispatcher { contract_address };
            hello_starknet.example_function();
        }
    }
}
