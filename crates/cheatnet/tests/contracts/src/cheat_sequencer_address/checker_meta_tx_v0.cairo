#[starknet::interface]
trait ICheatSequencerAddressCheckerMetaTxV0<TContractState> {
    fn __execute__(ref self: TContractState) -> felt252;
}

#[starknet::contract(account)]
mod CheatSequencerAddressCheckerMetaTxV0 {
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ICheatSequencerAddressCheckerMetaTxV0 of super::ICheatSequencerAddressCheckerMetaTxV0<
        ContractState,
    > {
        fn __execute__(ref self: ContractState) -> felt252 {
            let sequencer_address = starknet::get_block_info().unbox().sequencer_address;
            sequencer_address.into()
        }
    }
}
