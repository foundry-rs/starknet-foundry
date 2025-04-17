#[starknet::interface]
pub trait ITop<TContractState> {
    fn call_panic_contract(
        self: @TContractState, panic_contract_address: starknet::ContractAddress,
    );
}

#[feature("safe_dispatcher")]
#[starknet::contract]
mod Top {
    use super::{INestedSafeDispatcher, INestedSafeDispatcherTrait};

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl TopImpl of super::ITop<ContractState> {
        fn call_panic_contract(
            self: @ContractState, panic_contract_address: starknet::ContractAddress,
        ) {
            let dispatcher = INestedSafeDispatcher { contract_address: panic_contract_address };

            match dispatcher.do_panic() {
                Result::Ok(_) => core::panic_with_felt252('Expected panic'),
                Result::Err(err_data) => {
                    assert(*err_data.at(0) == 'Panic in Nested contract', 'Incorrect error');
                },
            }
        }
    }
}

#[starknet::interface]
pub trait INested<TContractState> {
    fn do_panic(self: @TContractState);
}


#[starknet::contract]
mod Nested {
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl NestedImpl of super::INested<ContractState> {
        fn do_panic(self: @ContractState) {
            core::panic_with_felt252('Panic in Nested contract');
        }
    }
}
