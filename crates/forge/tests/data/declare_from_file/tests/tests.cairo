use core::array::ArrayTrait;
use core::result::ResultTrait;
use snforge_std::{ContractClassTrait, DeclareResult, DeclareResultTrait, declare_from_file};

#[test]
fn simple() {
    let contract_class = declare_from_file(
        "target/dev/declare_from_file_HelloStarknet.contract_class.json",
    )
        .unwrap()
        .contract_class();

    contract_class.deploy(@ArrayTrait::new()).unwrap();
}

#[test]
fn already_declared() {
    declare_from_file("target/dev/declare_from_file_HelloStarknet.contract_class.json").unwrap();
    let result = declare_from_file("target/dev/declare_from_file_HelloStarknet.contract_class.json")
        .unwrap();

    match result {
        DeclareResult::AlreadyDeclared(_) => (),
        _ => panic!("expected AlreadyDeclared"),
    }
}

#[test]
fn missing_file() {
    declare_from_file("data/missing.contract_class.json").unwrap();
}

#[test]
fn invalid_json() {
    declare_from_file("data/invalid_contract_class.json").unwrap();
}
