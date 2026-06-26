#[starknet::contract]
pub mod HelloStarknet {
    #[storage]
    struct Storage {
        counter: felt252,
    }
}
