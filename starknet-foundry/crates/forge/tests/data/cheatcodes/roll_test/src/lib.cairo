
#[starknet::interface]
trait IRollChecker<TContractState> {
    fn is_rolled(ref self: TContractState, expected_block_number: u64);
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
        fn is_rolled(ref self: ContractState, expected_block_number: u64) {
            let block_numb = starknet::get_block_info().unbox().block_number;
            assert(block_numb == expected_block_number, 'block_numb incorrect');
        }
    }
}

#[starknet::interface]
trait IConstructorRollChecker<TContractState> {
    fn was_rolled_on_construction(ref self: TContractState, expected_block_number: u64);
}

#[starknet::contract]
mod ConstructorRollChecker {
    use box::BoxTrait;
    #[storage]
    struct Storage {
        blk_nb: u64,
    }

    #[constructor]
    fn constructor(ref self: ContractState) {
        let block_numb = starknet::get_block_info().unbox().block_number;
        self.blk_nb.write(block_numb);
    }

    #[external(v0)]
    impl IConstructorRollChecker of super::IConstructorRollChecker<ContractState> {
        fn was_rolled_on_construction(ref self: ContractState, expected_block_number: u64) {
            assert(self.blk_nb.read() == expected_block_number, 'block_numb incorrect');
        }
    }
}