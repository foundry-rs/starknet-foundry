use starknet::ContractAddress;

#[starknet::interface]
trait IElectChecker<TContractState> {
    fn get_sequencer_address(self: @TContractState) -> ContractAddress;
}

#[starknet::interface]
trait IElectCheckerProxy<TContractState> {
    fn get_elect_checkers_seq_addr(
        self: @TContractState, address: ContractAddress
    ) -> ContractAddress;
    fn get_sequencer_address(self: @TContractState) -> ContractAddress;
    fn call_proxy(
        self: @TContractState, address: ContractAddress
    ) -> (ContractAddress, ContractAddress);
}

#[starknet::contract]
mod ElectCheckerProxy {
    use starknet::ContractAddress;
    use super::IElectCheckerDispatcherTrait;
    use super::IElectCheckerDispatcher;
    use super::IElectCheckerProxyDispatcherTrait;
    use super::IElectCheckerProxyDispatcher;
    use starknet::get_contract_address;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IElectCheckerProxy of super::IElectCheckerProxy<ContractState> {
        fn get_elect_checkers_seq_addr(
            self: @ContractState, address: ContractAddress
        ) -> ContractAddress {
            let elect_checker = IElectCheckerDispatcher { contract_address: address };
            elect_checker.get_sequencer_address()
        }

        fn get_sequencer_address(self: @ContractState) -> ContractAddress {
            starknet::get_block_info().unbox().sequencer_address
        }

        fn call_proxy(
            self: @ContractState, address: ContractAddress
        ) -> (ContractAddress, ContractAddress) {
            let dispatcher = IElectCheckerProxyDispatcher { contract_address: address };
            let sequencer_address = self.get_sequencer_address();
            let res = dispatcher.get_elect_checkers_seq_addr(get_contract_address());
            (sequencer_address, res)
        }
    }
}
