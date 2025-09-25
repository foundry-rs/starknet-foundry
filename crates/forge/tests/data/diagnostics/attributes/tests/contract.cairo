use core::array::ArrayTrait;
use core::result::ResultTrait;
use snforge_std::cheatcodes::contract_class::DeclareResultTrait;
use snforge_std::{ContractClassTrait, declare};

@attrs@
fn call_and_invoke1(_a: felt252, b: u256) {
    let contract = declare("HelloStarknet").unwrap().contract_class();
    let constructor_calldata = @ArrayTrait::new();
    let (_contract_address1, _) = contract.deploy(constructor_calldata).unwrap()
}
