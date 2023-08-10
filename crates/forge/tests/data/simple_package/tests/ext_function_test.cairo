use result::ResultTrait;
use cheatcodes::{ declare, ContractClass, ContractClassTrait };
use array::ArrayTrait;
use traits::Into;
use traits::TryInto;
use starknet::ContractAddressIntoFelt252;
use debug::PrintTrait;
    
#[test]
fn deploy_invalid_calldata() {
    let mut calldata = ArrayTrait::new();

    let contract = declare('HelloStarknet');
    let contract_address_pre = contract.precalculate_address(@calldata);
    let contract_address = contract.deploy(@calldata).unwrap();
    let contract_address_pre2 = contract.precalculate_address(@calldata);
    let contract_address2 = contract.deploy(@calldata).unwrap();
    contract_address_pre.print();
    contract_address.print();
    assert(contract_address_pre == contract_address, contract_address.into());
    assert(contract_address_pre2 == contract_address2, contract_address.into());
}