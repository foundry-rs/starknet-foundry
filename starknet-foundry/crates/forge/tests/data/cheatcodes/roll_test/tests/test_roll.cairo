use result::ResultTrait;
use array::ArrayTrait;
use option::OptionTrait;
use traits::TryInto;
use starknet::ContractAddress;
use starknet::Felt252TryIntoContractAddress;
use cheatcodes::PreparedContract;
use forge_print::PrintTrait;

use roll_test::IRollCheckerDispatcher;
use roll_test::IRollCheckerDispatcherTrait;

use roll_test::IConstructorRollCheckerDispatcher;
use roll_test::IConstructorRollCheckerDispatcherTrait;

#[test]
fn test_roll_simple() {
    let class_hash = declare('RollChecker').unwrap();
    let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
    let contract_address = deploy(prepared).unwrap();
    let contract_address: ContractAddress = contract_address.try_into().unwrap();
    let dispatcher = IRollCheckerDispatcher { contract_address };

    start_roll(contract_address, 234);

    dispatcher.is_rolled(234);
}

// TODO: Failing
#[test]
fn test_roll_constructor_simple() {
    assert(1 == 1, 'simple check');
    let class_hash = declare('ConstructorRollChecker').unwrap();
    let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
    let contract_address: ContractAddress = 2598896470772924212281968896271340780432065735045468431712403008297614014532.try_into().unwrap();
    start_roll(contract_address, 234);
    let contract_address: ContractAddress = deploy(prepared).unwrap().try_into().unwrap();
    contract_address.print();

    let dispatcher = IConstructorRollCheckerDispatcher { contract_address };
    dispatcher.was_rolled_on_construction(234);
}
