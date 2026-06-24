use core::array::ArrayTrait;
use core::result::ResultTrait;
use declare_type_safe::hello_starknet::{
    HelloStarknet,
};
use snforge_std::cheatcodes::contract_class::DeclareResultTrait;

#[test]
fn declare_with_contract_name() {
    let _contract = declare!(HelloStarknet).unwrap().contract_class();
}

#[test]
fn declare_with_full_module_path() {
    let _contract = declare!(declare_type_safe::hello_starknet::HelloStarknet)
        .unwrap()
        .contract_class();
}

#[test]
fn declare_with_partial_module_path() {
    let _contract = declare!(hello_starknet::HelloStarknet)
        .unwrap()
        .contract_class();
}
