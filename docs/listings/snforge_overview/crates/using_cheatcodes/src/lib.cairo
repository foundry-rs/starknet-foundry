#[starknet::interface]
pub trait ICheatcodeChecker<TContractState> {
    fn increase_balance(ref self: TContractState, amount: felt252);
    fn get_balance(self: @TContractState) -> felt252;
    fn get_block_number_at_construction(self: @TContractState) -> u64;
    fn get_block_timestamp_at_construction(self: @TContractState) -> u64;
}

#[starknet::contract]
pub mod CheatcodeChecker {
    use core::box::BoxTrait;
    use starknet::get_caller_address;

    #[storage]
    struct Storage {
        balance: felt252,
        blk_nb: u64,
        blk_timestamp: u64,
    }

    #[constructor]
    fn constructor(ref self: ContractState) {
        // store the current block number
        self.blk_nb.write(starknet::get_block_info().unbox().block_number);
        // store the current block timestamp
        self.blk_timestamp.write(starknet::get_block_info().unbox().block_timestamp);
    }

    #[abi(embed_v0)]
    impl ICheatcodeCheckerImpl of super::ICheatcodeChecker<ContractState> {
        // Increases the balance by the given amount
        fn increase_balance(ref self: ContractState, amount: felt252) {
            assert_is_allowed_user();
            self.balance.write(self.balance.read() + amount);
        }
        // Gets the balance.
        fn get_balance(self: @ContractState) -> felt252 {
            self.balance.read()
        }
        // Gets the block number
        fn get_block_number_at_construction(self: @ContractState) -> u64 {
            self.blk_nb.read()
        }
        // Gets the block timestamp
        fn get_block_timestamp_at_construction(self: @ContractState) -> u64 {
            self.blk_timestamp.read()
        }
    }

    fn assert_is_allowed_user() {
        // checks if caller is '123'
        let address = get_caller_address();
        assert(address.into() == 123, 'user is not allowed');
    }
}
