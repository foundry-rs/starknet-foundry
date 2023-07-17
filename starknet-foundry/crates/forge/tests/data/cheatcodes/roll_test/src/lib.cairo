
#[starknet::interface]
trait IRollChecker<TContractState> {
    fn is_rolled(ref self: TContractState, expected_block_number: u64) -> u64;
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
        fn is_rolled(ref self: ContractState, expected_block_number: u64) -> u64 {
            // let block_numb = get_block_info().unbox().block_number
            let block_numb = starknet::get_block_info().unbox().block_number;
            assert(block_numb == block_numb, 'block_numb incorrect');
            return block_numb;
        }
    }
}