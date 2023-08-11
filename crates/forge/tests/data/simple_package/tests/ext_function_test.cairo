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
    calldata.append('token');   // name
    calldata.append('TKN');     // symbol
    calldata.append(18);        // decimals
    calldata.append(1111);      // initial supply low
    calldata.append(0);         // initial supply high
    calldata.append(1234);      // recipient


    let contract = declare('ERC20');
    let contract_address_pre = contract.precalculate_address(@calldata);
    let contract_address = contract.deploy(@calldata).unwrap();
 
    contract_address_pre.print();
    contract_address.print();

    assert(contract_address_pre == contract_address, contract_address.into());
    // assert(contract_address_pre2 == contract_address2, contract_address.into());
}