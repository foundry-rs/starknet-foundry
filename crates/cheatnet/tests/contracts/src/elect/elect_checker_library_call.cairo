use starknet::{ClassHash, ContractAddress};

#[starknet::interface]
trait IElectChecker<TContractState> {
    fn get_sequencer_address(ref self: TContractState) -> ContractAddress;
}

#[starknet::interface]
trait IElectCheckerLibCall<TContractState> {
    fn get_sequencer_address_with_lib_call(
        ref self: TContractState, class_hash: ClassHash
    ) -> ContractAddress;
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
            ref self: ContractState, class_hash: ClassHash
        ) -> ContractAddress {
            let elect_checker = IElectCheckerLibraryDispatcher { class_hash };
            elect_checker.get_sequencer_address()
        }
    }
}
