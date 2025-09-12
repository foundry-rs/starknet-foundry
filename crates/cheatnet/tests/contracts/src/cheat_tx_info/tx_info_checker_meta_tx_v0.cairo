#[starknet::interface]
trait ITxInfoCheckerMetaTxV0<TContractState> {
    fn __execute__(ref self: TContractState) -> felt252;
}

#[starknet::contract(account)]
mod TxInfoCheckerMetaTxV0 {
    use starknet::get_execution_info;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ITxInfoCheckerMetaTxV0 of super::ITxInfoCheckerMetaTxV0<ContractState> {
        fn __execute__(ref self: ContractState) -> felt252 {
            let execution_info = get_execution_info().unbox();
            let tx_info = execution_info.tx_info.unbox();
            tx_info.version
        }
    }
}
