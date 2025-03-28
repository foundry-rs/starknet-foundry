use starknet::{ClassHash, ContractAddress};

#[starknet::interface]
trait IHelloStarknet<TContractState> {
    fn get_balance(self: @TContractState) -> felt252;
}

#[starknet::interface]
trait IForkingChecker<TContractState> {
    fn get_balance_call_contract(
        ref self: TContractState, contract_address: ContractAddress,
    ) -> felt252;
    fn get_balance_library_call(ref self: TContractState, class_hash: ClassHash) -> felt252;
    fn set_balance(ref self: TContractState, new_balance: felt252);
}

#[starknet::contract]
mod ForkingChecker {
    use super::{
        IHelloStarknetDispatcherTrait, IHelloStarknetDispatcher, IHelloStarknetLibraryDispatcher,
    };
    use starknet::{ClassHash, ContractAddress, OptionTrait};

    #[storage]
    struct Storage {
        balance: felt252,
    }

    #[constructor]
    fn constructor(ref self: ContractState, contract_address: Option<ContractAddress>) {
        if contract_address.is_some() {
            let hello_starknet = IHelloStarknetDispatcher {
                contract_address: contract_address.unwrap(),
            };
            self.balance.write(hello_starknet.get_balance());
        }
    }

    #[abi(embed_v0)]
    impl IForkingCheckerImpl of super::IForkingChecker<ContractState> {
        fn get_balance_call_contract(
            ref self: ContractState, contract_address: ContractAddress,
        ) -> felt252 {
            let hello_starknet = IHelloStarknetDispatcher { contract_address };
            hello_starknet.get_balance()
        }

        fn get_balance_library_call(ref self: ContractState, class_hash: ClassHash) -> felt252 {
            let hello_starknet = IHelloStarknetLibraryDispatcher { class_hash };
            hello_starknet.get_balance()
        }

        fn set_balance(ref self: ContractState, new_balance: felt252) {
            self.balance.write(new_balance);
        }
    }
}
