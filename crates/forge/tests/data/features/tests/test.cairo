use snforge_std::{declare, ContractClass, ContractClassTrait};
use features::{IContractDispatcher, IContractDispatcherTrait};

#[cfg(feature: 'snforge_test_only')]
fn mock_in_tests() -> felt252 {
    69
}

#[test]
#[cfg(feature: 'snforge_test_only')]
fn test_mock_function() {
    assert(mock_in_tests() == 69, '');
}

#[test]
fn test_mock_contract() {
    let (contract_address, _) = declare("MockContract").unwrap().deploy(@array![]).unwrap();
    let response_result = IContractDispatcher { contract_address }.response();
    assert(response_result == 420, '');
}
