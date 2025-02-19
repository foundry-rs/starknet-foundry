use starknet::ContractAddress;

#[starknet::interface]
trait IConstructorMockChecker<TContractState> {
    fn get_constructor_balance(ref self: TContractState) -> felt252;
}

#[starknet::contract]
mod ConstructorMockChecker {
    use starknet::ContractAddress;
    use core::starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};

    #[starknet::interface]
    trait IHelloStarknet<TContractState> {
        fn get_balance(self: @TContractState) -> felt252;
    }

    #[storage]
    struct Storage {
        constructor_balance: felt252
    }

    #[constructor]
    fn constructor(ref self: ContractState, balance_contract_address: ContractAddress) {
        let hello_starknet = IHelloStarknetDispatcher {
            contract_address: balance_contract_address
        };
        self.constructor_balance.write(hello_starknet.get_balance());
    }

    #[abi(embed_v0)]
    impl IConstructorMockCheckerImpl of super::IConstructorMockChecker<ContractState> {
        fn get_constructor_balance(ref self: ContractState) -> felt252 {
            self.constructor_balance.read()
        }
    }
}
