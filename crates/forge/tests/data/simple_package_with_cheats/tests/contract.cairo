use core::array::ArrayTrait;
use core::result::ResultTrait;
use simple_package_with_cheats::{IHelloStarknetDispatcher, IHelloStarknetDispatcherTrait};
use snforge_std::cheatcodes::contract_class::DeclareResultTrait;
use snforge_std::{ContractClassTrait, declare, start_cheat_block_number_global};

#[test]
fn call_and_invoke() {
    let contract = declare("HelloStarknet").unwrap().contract_class();
    let constructor_calldata = @ArrayTrait::new();
    let (contract_address, _) = contract.deploy(constructor_calldata).unwrap();
    let dispatcher = IHelloStarknetDispatcher { contract_address };

    let block_number = dispatcher.get_block_number();
    println!("block number {}", block_number);
    // TODO investigate why the default is 2000
    assert(block_number == 2000, 'block_info == 2000');

    start_cheat_block_number_global(123);

    let block_number = dispatcher.get_block_number();
    assert(block_number == 123, 'block_info == 123');
}
