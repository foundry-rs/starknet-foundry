use snforge_std::{declare, ContractClassTrait, DeclareResultTrait};

use conditional_compilation::contract::{IMockContractDispatcher, IMockContractDispatcherTrait};

#[test]
fn test_mock_contract() {
    let (contract_address, _) = declare("MockContract")
        .unwrap()
        .contract_class()
        .deploy(@array![])
        .unwrap();

    let dispatcher = IMockContractDispatcher { contract_address };
    let response = dispatcher.response();

    assert_eq!(response, 1);
}
