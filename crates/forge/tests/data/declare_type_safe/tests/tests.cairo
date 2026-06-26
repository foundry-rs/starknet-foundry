use core::array::ArrayTrait;
use core::result::ResultTrait;
use declare_type_safe::hello_starknet;
use declare_type_safe::hello_starknet as hello_starknet_alias;
use declare_type_safe::hello_starknet::HelloStarknet;
use declare_type_safe::hello_starknet::HelloStarknet as HelloStarknetAlias;
use snforge_std::cheatcodes::contract_class::DeclareResultTrait;

#[test]
fn declare_with_full_path() {
    let _contract = declare!(declare_type_safe::hello_starknet::HelloStarknet)
        .unwrap()
        .contract_class();
}

#[test]
fn declare_with_partial_path() {
    let _contract = declare!(hello_starknet::HelloStarknet).unwrap().contract_class();
}

#[test]
fn declare_with_contract_name() {
    let _contract = declare!(HelloStarknet).unwrap().contract_class();
}

#[test]
#[should_panic(expected: "Failed to get contract artifact for identifier = HelloStarknetAlias.")]
fn declare_with_contract_alias_is_not_resolved_as_canonical_path() {
    declare!(HelloStarknetAlias).unwrap();
}

#[test]
#[should_panic(
    expected: "Failed to get contract artifact for identifier = hello_starknet_alias::HelloStarknet."
)]
fn declare_with_module_alias_is_not_resolved_as_canonical_path() {
    declare!(hello_starknet_alias::HelloStarknet).unwrap();
}
