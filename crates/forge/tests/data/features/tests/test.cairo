use snforge_std::{declare, DeclareResultTrait, ContractClass, ContractClassTrait};
use features::{IContractDispatcher, IContractDispatcherTrait};

#[cfg(feature: 'snforge_test_only')]
fn mock_in_tests() -> felt252 {
    999
}

#[test]
fn test_mock_function() {
    assert(mock_in_tests() == 999, '');
}

#[test]
fn test_mock_contract() {
    let (contract_address, _) = declare("MockContract")
        .unwrap()
        .contract_class()
        .deploy(@array![])
        .unwrap();
    let response_result = IContractDispatcher { contract_address }.response();
    assert(response_result == 1234, '');
}
