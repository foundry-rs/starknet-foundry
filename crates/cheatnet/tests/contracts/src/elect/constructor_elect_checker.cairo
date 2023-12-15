use starknet::ContractAddress;

#[starknet::interface]
trait IConstructorElectChecker<TContractState> {
    fn get_stored_sequencer_address(ref self: TContractState) -> ContractAddress;
}

#[starknet::contract]
mod ConstructorElectChecker {
    use box::BoxTrait;
    use starknet::ContractAddress;

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
    impl IConstructorElectChecker of super::IConstructorElectChecker<ContractState> {
        fn get_stored_sequencer_address(ref self: ContractState) -> ContractAddress {
            self.seq_addr.read()
        }
    }
}
