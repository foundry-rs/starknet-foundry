#[starknet::contract]
pub mod HelloStarknet {
    #[storage]
    struct Storage {
        balance: felt252,
    }
}
