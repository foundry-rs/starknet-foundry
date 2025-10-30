#[starknet::interface]
trait IContract<TContractState> {
    /// Write `value` to storage and then panic
    fn write_storage_and_panic(ref self: TContractState, value: felt252);
    /// Return storage value
    fn read_storage(self: @TContractState) -> felt252;
    /// Write `value` to storage and emits event
    fn write_storage(ref self: TContractState, value: felt252);
}

#[starknet::contract]
mod Contract {
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};

    #[storage]
    struct Storage {
        value: felt252,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        ValueUpdated: ValueUpdated,
    }

    #[derive(Drop, starknet::Event)]
    pub struct ValueUpdated {
        pub old_value: felt252,
        pub new_value: felt252,
    }

    #[abi(embed_v0)]
    impl IContractImpl of super::IContract<ContractState> {
        fn write_storage_and_panic(ref self: ContractState, value: felt252) {
            self.value.write(value);
            panic!("Panicked");
        }

        fn read_storage(self: @ContractState) -> felt252 {
            self.value.read()
        }

        fn write_storage(ref self: ContractState, value: felt252) {
            let old_value = self.value.read();
            self.emit(ValueUpdated { old_value, new_value: value});
            self.value.write(value);
        }
    }
}
