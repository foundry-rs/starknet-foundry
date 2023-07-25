#[starknet::interface]
trait IRollChecker<TContractState> {
    fn get_block_number(ref self: TContractState) -> u64;
}

#[starknet::contract]
mod RollChecker {
    use box::BoxTrait;
    #[storage]
    struct Storage {
        balance: felt252,
    }

    #[external(v0)]
    impl IRollChecker of super::IRollChecker<ContractState> {
        fn get_block_number(ref self: ContractState) -> u64 {
            starknet::get_block_info().unbox().block_number
        }
    }
}
