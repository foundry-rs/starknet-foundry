use starknet::{ClassHash, ContractAddress};

#[starknet::interface]
trait ICheatSequencerAddressChecker<TContractState> {
    fn get_sequencer_address(self: @TContractState) -> ContractAddress;
}

#[starknet::interface]
trait ICheatSequencerAddressCheckerLibCall<TContractState> {
    fn get_sequencer_address_with_lib_call(
        self: @TContractState, class_hash: ClassHash,
    ) -> ContractAddress;
    fn get_sequencer_address(self: @TContractState) -> ContractAddress;
}

#[starknet::contract]
mod CheatSequencerAddressCheckerLibCall {
    use super::{
        ICheatSequencerAddressCheckerDispatcherTrait,
        ICheatSequencerAddressCheckerLibraryDispatcher,
    };
    use starknet::{ClassHash, ContractAddress};

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ICheatSequencerAddressCheckerLibCall of super::ICheatSequencerAddressCheckerLibCall<
        ContractState,
    > {
        fn get_sequencer_address_with_lib_call(
            self: @ContractState, class_hash: ClassHash,
        ) -> ContractAddress {
            let sequencer_address_checker = ICheatSequencerAddressCheckerLibraryDispatcher {
                class_hash,
            };
            sequencer_address_checker.get_sequencer_address()
        }

        fn get_sequencer_address(self: @ContractState) -> ContractAddress {
            starknet::get_block_info().unbox().sequencer_address
        }
    }
}
