use core::box::BoxTrait;
use core::result::ResultTrait;
use snforge_std::{start_cheat_block_number, stop_cheat_block_number, test_address};
use starknet::ContractAddress;

#[test]
fn test_cheat_block_number_test_state() {
    let test_address: ContractAddress = test_address();
    let old_block_number = starknet::get_block_info().unbox().block_number;

    start_cheat_block_number(test_address, 234);
    let new_block_number = starknet::get_block_info().unbox().block_number;
    assert(new_block_number == 234, 'Wrong block number');

    stop_cheat_block_number(test_address);
    let new_block_number = starknet::get_block_info().unbox().block_number;
    assert(new_block_number == old_block_number, 'Block num did not change back');
}
