use snforge_std::{ContractClassTrait, DeclareResultTrait, declare};
use test_case::{IValueStorageDispatcher, IValueStorageDispatcherTrait};

#[test]
#[test_case(100)]
#[test_case(42)]
#[test_case(0)]
fn with_contract_deploy(value: u128) {
    let contract = declare("ValueStorage").unwrap().contract_class();
    let (contract_address, _) = contract.deploy(@array![]).unwrap();
    let dispatcher = IValueStorageDispatcher { contract_address };

    dispatcher.set_value(value);
    assert!(dispatcher.get_value() == value, "Value mismatch");
}

#[test]
#[fuzzer]
#[test_case(123)]
#[test_case(0)]
fn with_fuzzer_and_contract_deploy(value: u128) {
    let contract = declare("ValueStorage").unwrap().contract_class();
    let (contract_address, _) = contract.deploy(@array![]).unwrap();
    let dispatcher = IValueStorageDispatcher { contract_address };

    dispatcher.set_value(value);
    assert!(dispatcher.get_value() == value, "FAIL");
}

