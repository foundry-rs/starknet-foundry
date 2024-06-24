use starknet::{ClassHash, ContractAddress};

#[starknet::interface]
trait IElectChecker<TContractState> {
    fn get_sequencer_address(self: @TContractState) -> ContractAddress;
}

#[starknet::interface]
trait IElectCheckerLibCall<TContractState> {
    fn get_sequencer_address_with_lib_call(
        self: @TContractState, class_hash: ClassHash
    ) -> ContractAddress;
    fn get_sequencer_address(self: @TContractState) -> ContractAddress;
}

#[starknet::contract]
mod ElectCheckerLibCall {
    use super::{IElectCheckerDispatcherTrait, IElectCheckerLibraryDispatcher};
    use starknet::{ClassHash, ContractAddress};

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IElectCheckerLibCall of super::IElectCheckerLibCall<ContractState> {
        fn get_sequencer_address_with_lib_call(
            self: @ContractState, class_hash: ClassHash
        ) -> ContractAddress {
            let elect_checker = IElectCheckerLibraryDispatcher { class_hash };
            elect_checker.get_sequencer_address()
        }

        fn get_sequencer_address(self: @ContractState) -> ContractAddress {
            starknet::get_block_info().unbox().sequencer_address
        }
    }
}
