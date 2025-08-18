#[starknet::interface]
trait ICheatBlockNumberCheckerMetaTxV0<TContractState> {
    fn __execute__(ref self: TContractState) -> felt252;
    fn __validate__(ref self: TContractState) -> felt252;
}

#[starknet::contract(account)]
mod CheatBlockNumberCheckerMetaTxV0 {
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ICheatBlockNumberCheckerMetaTxV0 of super::ICheatBlockNumberCheckerMetaTxV0<
        ContractState,
    > {
        fn __execute__(ref self: ContractState) -> felt252 {
            let block_number = starknet::get_block_number();
            block_number.into()
        }

        fn __validate__(ref self: ContractState) -> felt252 {
            starknet::VALIDATED
        }
    }
}
