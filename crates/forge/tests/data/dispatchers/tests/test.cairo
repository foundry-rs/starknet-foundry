use dispatchers::error_handler::{
    IErrorHandlerDispatcher, IErrorHandlerDispatcherTrait, IErrorHandlerSafeDispatcher,
    IErrorHandlerSafeDispatcherTrait,
};
use snforge_std::{declare, DeclareResultTrait, ContractClassTrait};
use starknet::ContractAddress;

fn deploy_contracts() -> ContractAddress {
    let failable = declare("FailableContract").unwrap().contract_class();
    let (failable_address, _) = failable.deploy(@array![]).unwrap();

    let error_handler = declare("ErrorHandler").unwrap().contract_class();
    let (contract_address, _) = error_handler.deploy(@array![failable_address.into()]).unwrap();
    contract_address
}


#[test]
fn test_error_handled_in_contract() {
    let contract_address = deploy_contracts();

    let dispatcher = IErrorHandlerDispatcher { contract_address };

    dispatcher.catch_panic_and_handle();
}

#[should_panic(expected: 'Different panic')]
#[test]
fn test_handle_and_panic() {
    let contract_address = deploy_contracts();

    let dispatcher = IErrorHandlerDispatcher { contract_address };

    dispatcher.catch_panic_and_fail();
}

#[feature("safe_dispatcher")]
#[test]
fn test_handle_recoverable_in_test() {
    let contract_address = deploy_contracts();

    let dispatcher = IErrorHandlerSafeDispatcher { contract_address };

    match dispatcher.catch_panic_and_fail() {
        Result::Ok(_) => core::panic_with_felt252('Expected panic'),
        Result::Err(panic_data) => {
            assert(*panic_data.at(0) == 'Different panic', 'Incorrect error');
            assert(panic_data.len() == 1, 'Incorrect error length');
        },
    }
}

#[feature("safe_dispatcher")]
#[test]
fn test_unrecoverable_not_possible_to_handle() {
    let contract_address = deploy_contracts();

    let dispatcher = IErrorHandlerSafeDispatcher { contract_address };

    match dispatcher.call_unrecoverable() {
        // Unreachable
        Result::Ok(_) => {},
        Result::Err(_) => {},
    }
}
