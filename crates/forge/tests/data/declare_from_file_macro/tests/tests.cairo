use core::array::ArrayTrait;
use core::result::ResultTrait;
use snforge_std::{ContractClassTrait, DeclareResult, DeclareResultTrait};

#[test]
fn declare_with_sierra_file_path() {
    declare_from_file!("target/dev/declare_from_file_macro_HelloStarknet.contract_class.json")
        .unwrap();
}

#[test]
fn declare_from_file_already_declared() {
    declare_from_file!("target/dev/declare_from_file_macro_HelloStarknet.contract_class.json")
        .unwrap();
    let result = declare_from_file!(
        "target/dev/declare_from_file_macro_HelloStarknet.contract_class.json",
    )
        .unwrap();

    match result {
        DeclareResult::AlreadyDeclared(_) => (),
        _ => panic!("expected AlreadyDeclared"),
    }
}

#[test]
fn declare_from_file_contract_class_can_be_deployed() {
    let contract_class = declare_from_file!(
        "target/dev/declare_from_file_macro_HelloStarknet.contract_class.json",
    )
        .unwrap()
        .contract_class();

    contract_class.deploy(@ArrayTrait::new()).unwrap();
}
