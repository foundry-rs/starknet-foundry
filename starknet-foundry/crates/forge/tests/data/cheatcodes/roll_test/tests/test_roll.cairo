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

#[test]
fn test_roll_simple() {
    assert(1 == 1, 'simple check');
    let class_hash = declare('RollChecker').unwrap();
    let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
    let contract_address = deploy(prepared).unwrap();
    let contract_address: ContractAddress = contract_address.try_into().unwrap();
    let dispatcher = IRollCheckerDispatcher { contract_address };

    dispatcher.is_rolled(123);
}



// #[test]
// fn test_roll_constructor_simple() { // TODO
//     assert(1 == 1, 'simple check');
//     let class_hash = declare('HelloStarknet').unwrap();
//     let prepared = PreparedContract { contract_address: 1234, class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
//     let contract_address = deploy(prepared).unwrap();
// }



