#[starknet::contract]
mod HelloStarknet {
    #[storage]
    struct Storage {
        balance: felt252,
    }
}

#[starknet::contract]
mod ERC20 {
    #[storage]
    struct Storage {
        balance: felt252,
    }
}
