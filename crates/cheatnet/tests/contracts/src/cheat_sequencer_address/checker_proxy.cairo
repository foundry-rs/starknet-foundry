use starknet::ContractAddress;

#[starknet::interface]
trait ICheatSequencerAddressChecker<TContractState> {
    fn get_sequencer_address(self: @TContractState) -> ContractAddress;
}

#[starknet::interface]
trait ICheatSequencerAddressCheckerProxy<TContractState> {
    fn get_cheated_sequencer_address(
        self: @TContractState, address: ContractAddress
    ) -> ContractAddress;
    fn get_sequencer_address(self: @TContractState) -> ContractAddress;
    fn call_proxy(
        self: @TContractState, address: ContractAddress
    ) -> (ContractAddress, ContractAddress);
}

#[starknet::contract]
mod CheatSequencerAddressCheckerProxy {
    use starknet::ContractAddress;
    use super::ICheatSequencerAddressCheckerDispatcherTrait;
    use super::ICheatSequencerAddressCheckerDispatcher;
    use super::ICheatSequencerAddressCheckerProxyDispatcherTrait;
    use super::ICheatSequencerAddressCheckerProxyDispatcher;
    use starknet::get_contract_address;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ICheatSequencerAddressCheckerProxy of super::ICheatSequencerAddressCheckerProxy<
        ContractState
    > {
        fn get_cheated_sequencer_address(
            self: @ContractState, address: ContractAddress
        ) -> ContractAddress {
            let sequencer_address_checker = ICheatSequencerAddressCheckerDispatcher {
                contract_address: address
            };
            sequencer_address_checker.get_sequencer_address()
        }

        fn get_sequencer_address(self: @ContractState) -> ContractAddress {
            starknet::get_block_info().unbox().sequencer_address
        }

        fn call_proxy(
            self: @ContractState, address: ContractAddress
        ) -> (ContractAddress, ContractAddress) {
            let dispatcher = ICheatSequencerAddressCheckerProxyDispatcher {
                contract_address: address
            };
            let sequencer_address = self.get_sequencer_address();
            let res = dispatcher.get_cheated_sequencer_address(get_contract_address());
            (sequencer_address, res)
        }
    }
}
