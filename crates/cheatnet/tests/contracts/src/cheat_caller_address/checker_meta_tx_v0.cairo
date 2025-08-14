#[starknet::interface]
trait ICheatCallerAddressCheckerMetaTxV0<TContractState> {
    fn __execute__(ref self: TContractState) -> felt252;
    fn __validate__(ref self: TContractState) -> felt252;
}

#[starknet::contract(account)]
mod CheatCallerAddressCheckerMetaTxV0 {
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ICheatCallerAddressCheckerMetaTxV0 of super::ICheatCallerAddressCheckerMetaTxV0<ContractState> {
        fn __execute__(ref self: ContractState) -> felt252 {
            starknet::get_caller_address().into()
        }

        fn __validate__(ref self: ContractState) -> felt252 {
            starknet::VALIDATED
        }
    }
}
