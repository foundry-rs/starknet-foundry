#[starknet::contract]
mod SequencerAddressChecker {
    use result::ResultTrait;
    use starknet::{ ClassHash, library_call_syscall, ContractAddress};

    #[storage]
    struct Storage {
    }

    #[external(v0)]
    fn get_sequencer_address(ref self: ContractState) -> ContractAddress {
        starknet::get_block_info().unbox().sequencer_address
    }
}