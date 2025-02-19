#[starknet::interface]
trait ITimestamper<TContractState> {
    fn write_timestamp(ref self: TContractState);
    fn read_timestamp(self: @TContractState) -> u64;
}

#[starknet::contract]
mod Timestamper {
    use core::array::ArrayTrait;
    use starknet::get_block_timestamp;
    use core::starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};

    #[storage]
    struct Storage {
        time: u64,
    }

    #[abi(embed_v0)]
    impl ITimestamperImpl of super::ITimestamper<ContractState> {
        fn write_timestamp(ref self: ContractState) {
            let time = get_block_timestamp();
            self.time.write(time);
        }

        fn read_timestamp(self: @ContractState) -> u64 {
            self.time.read()
        }
    }
}
