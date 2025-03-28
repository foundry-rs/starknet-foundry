use starknet::ContractAddress;

#[starknet::interface]
trait IConstructorCheatSequencerAddressChecker<TContractState> {
    fn get_stored_sequencer_address(ref self: TContractState) -> ContractAddress;
    fn get_sequencer_address(self: @TContractState) -> ContractAddress;
}

#[starknet::contract]
mod ConstructorCheatSequencerAddressChecker {
    use core::box::BoxTrait;
    use starknet::ContractAddress;
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};

    #[storage]
    struct Storage {
        seq_addr: ContractAddress,
    }

    #[constructor]
    fn constructor(ref self: ContractState) {
        let sequencer_address = starknet::get_block_info().unbox().sequencer_address;
        self.seq_addr.write(sequencer_address);
    }

    #[abi(embed_v0)]
    impl IConstructorCheatSequencerAddressChecker of super::IConstructorCheatSequencerAddressChecker<
        ContractState,
    > {
        fn get_stored_sequencer_address(ref self: ContractState) -> ContractAddress {
            self.seq_addr.read()
        }

        fn get_sequencer_address(self: @ContractState) -> ContractAddress {
            starknet::get_block_info().unbox().sequencer_address
        }
    }
}
