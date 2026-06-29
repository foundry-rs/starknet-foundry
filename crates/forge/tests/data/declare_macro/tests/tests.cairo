use core::array::ArrayTrait;
use core::result::ResultTrait;
use declare_macro::hello_starknet::{HelloStarknet, HelloStarknet as HelloStarknetAlias};
use declare_macro::{hello_starknet, hello_starknet as hello_starknet_alias};

#[test]
fn declare_with_full_path() {
    declare!(declare_macro::hello_starknet::HelloStarknet).unwrap();
}

#[test]
fn declare_with_partial_path() {
    declare!(hello_starknet::HelloStarknet).unwrap();
}

#[test]
fn declare_with_contract_name() {
    declare!(HelloStarknet).unwrap();
}

#[test]
#[should_panic(expected: "Failed to get contract artifact for identifier = HelloStarknetAlias.")]
fn declare_with_contract_alias_is_not_resolved_as_canonical_path() {
    declare!(HelloStarknetAlias).unwrap();
}

#[test]
#[should_panic(
    expected: "Failed to get contract artifact for identifier = hello_starknet_alias::HelloStarknet.",
)]
fn declare_with_module_alias_is_not_resolved_as_canonical_path() {
    declare!(hello_starknet_alias::HelloStarknet).unwrap();
}
