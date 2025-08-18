#[starknet::interface]
trait ICheatBlockTimestampCheckerMetaTxV0<TContractState> {
    fn __execute__(ref self: TContractState) -> felt252;
    fn __validate__(ref self: TContractState) -> felt252;
}

#[starknet::contract(account)]
mod CheatBlockTimestampCheckerMetaTxV0 {
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ICheatBlockTimestampCheckerMetaTxV0 of super::ICheatBlockTimestampCheckerMetaTxV0<
        ContractState,
    > {
        fn __execute__(ref self: ContractState) -> felt252 {
            let block_timestamp = starknet::get_block_timestamp();
            block_timestamp.into()
        }

        fn __validate__(ref self: ContractState) -> felt252 {
            starknet::VALIDATED
        }
    }
}
