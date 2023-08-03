#[contract]
mod Map {
    struct Storage {
        storage: LegacyMap::<felt252, felt252>,
    }

    // Puts value in the storage
    #[external]
    fn put2(key:felt252, value: felt252) {
        storage::write(key, value);
    }

    // Returns value at the key
    #[view]
    fn get(key: felt252) -> felt252 {
        storage::read(key)
    }
}
