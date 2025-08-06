use snforge_std::{ContractClassTrait, DeclareResultTrait, declare};

pub fn deploy_contract(name: ByteArray, calldata: Array<felt252>) -> starknet::ContractAddress {
    let contract = declare(name).unwrap().contract_class();

    let (contract_address, _) = contract.deploy(@calldata).unwrap();
    contract_address
}
