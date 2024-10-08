#[starknet::interface]
pub trait ISimpleStorageContract<TState> {}

#[starknet::contract]
mod SimpleStorageContract {
    use starknet::storage::{Map, StoragePathEntry, StoragePointerWriteAccess};

    #[storage]
    struct Storage {
        plain_felt: felt252,
        mapping: Map<felt252, felt252>,
    }

    #[constructor]
    fn constructor(ref self: ContractState) {
        self.plain_felt.write(0x2137_felt252);
        self.mapping.entry('some_key').write('some_value');
    }
}
