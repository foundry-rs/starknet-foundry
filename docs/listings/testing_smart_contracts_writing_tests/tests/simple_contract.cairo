use snforge_std::{declare, ContractClassTrait, DeclareResultTrait};

use testing_smart_contracts_writing_tests::{
    ISimpleContractDispatcher, ISimpleContractDispatcherTrait
};

#[test]
fn call_and_invoke() {
    // First declare and deploy a contract
    let contract = declare("SimpleContract").unwrap().contract_class();
    // Alternatively we could use `deploy_syscall` here
    let (contract_address, _) = contract.deploy(@array![]).unwrap();

    // Create a Dispatcher object that will allow interacting with the deployed contract
    let dispatcher = ISimpleContractDispatcher { contract_address };

    // Call a view function of the contract
    let balance = dispatcher.get_balance();
    assert(balance == 0, 'balance == 0');

    // Call a function of the contract
    // Here we mutate the state of the storage
    dispatcher.increase_balance(100);

    // Check that transaction took effect
    let balance = dispatcher.get_balance();
    assert(balance == 100, 'balance == 100');
}
