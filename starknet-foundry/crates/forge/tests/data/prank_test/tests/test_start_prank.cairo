use protostar_print::PrintTrait;
use array::ArrayTrait;
use result::ResultTrait;
use option::OptionTrait;
use traits::TryInto;
use starknet::Felt252TryIntoContractAddress;
use starknet::ContractAddress;
use cheatcodes::PreparedContract;


#[test]
fn test_prank_basic() {
    let class_hash = declare('PrankedContract').unwrap();
    let prepared = PreparedContract { contract_address: 4567, class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
    let contract_address = deploy(prepared).unwrap();

    let return_data = call(contract_address, 'return_callers_address', @ArrayTrait::new()).unwrap();
    let callers_address_before = *return_data.at(0_u32);

    start_prank(123, contract_address).unwrap();

    let return_data = call(contract_address, 'return_callers_address', @ArrayTrait::new()).unwrap();
    let callers_address_after = *return_data.at(0_u32);

}
