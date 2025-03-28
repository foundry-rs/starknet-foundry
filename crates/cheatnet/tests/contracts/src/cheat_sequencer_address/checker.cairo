use starknet::ContractAddress;

#[starknet::interface]
trait ICheatSequencerAddressChecker<TContractState> {
    fn get_sequencer_address(ref self: TContractState) -> ContractAddress;
    fn get_seq_addr_and_emit_event(ref self: TContractState) -> ContractAddress;
}

#[starknet::contract]
mod CheatSequencerAddressChecker {
    use starknet::ContractAddress;

    #[storage]
    struct Storage {}

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        SequencerAddressEmitted: SequencerAddressEmitted
    }

    #[derive(Drop, starknet::Event)]
    struct SequencerAddressEmitted {
        sequencer_address: ContractAddress
    }

    #[abi(embed_v0)]
    impl ICheatSequencerAddressChecker of super::ICheatSequencerAddressChecker<ContractState> {
        fn get_sequencer_address(ref self: ContractState) -> ContractAddress {
            starknet::get_block_info().unbox().sequencer_address
        }

        fn get_seq_addr_and_emit_event(ref self: ContractState) -> ContractAddress {
            let sequencer_address = starknet::get_block_info().unbox().sequencer_address;
            self
                .emit(
                    Event::SequencerAddressEmitted(SequencerAddressEmitted { sequencer_address })
                );
            sequencer_address
        }
    }
}
