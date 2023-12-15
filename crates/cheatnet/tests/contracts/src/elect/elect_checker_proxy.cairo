use starknet::ContractAddress;

#[starknet::interface]
trait IElectChecker<TContractState> {
    fn get_sequencer_address(ref self: TContractState) -> ContractAddress;
}

#[starknet::interface]
trait IElectCheckerProxy<TContractState> {
    fn get_elect_checkers_seq_addr(
        ref self: TContractState, address: ContractAddress
    ) -> ContractAddress;
}

#[starknet::contract]
mod ElectCheckerProxy {
    use starknet::ContractAddress;
    use super::IElectCheckerDispatcherTrait;
    use super::IElectCheckerDispatcher;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IElectCheckerProxy of super::IElectCheckerProxy<ContractState> {
        fn get_elect_checkers_seq_addr(
            ref self: ContractState, address: ContractAddress
        ) -> ContractAddress {
            let elect_checker = IElectCheckerDispatcher { contract_address: address };
            elect_checker.get_sequencer_address()
        }
    }
}
