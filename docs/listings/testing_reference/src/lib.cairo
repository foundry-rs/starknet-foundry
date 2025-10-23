#[starknet::interface]
pub trait ICounter<TContractState> {
    fn increment(ref self: TContractState);
}

#[starknet::contract]
pub mod Counter {
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};

    #[storage]
    struct Storage {
        i: felt252,
    }

    #[constructor]
    fn constructor(ref self: ContractState) {
        self.i.write(0);
    }

    #[abi(embed_v0)]
    impl CounterImpl of super::ICounter<ContractState> {
        fn increment(ref self: ContractState) {
            let current_value = self.i.read();
            self.i.write(current_value + 1);
        }
    }
}
