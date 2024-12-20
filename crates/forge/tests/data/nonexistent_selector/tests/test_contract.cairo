use snforge_std::{declare, ContractClassTrait, DeclareResultTrait};

use nonexistent_selector::IMyContractSafeDispatcher;
use nonexistent_selector::IMyContractSafeDispatcherTrait;

#[test]
#[feature("safe_dispatcher")]
fn test_unwrapped_call_contract_syscall() {
    let contract = declare("MyContract").unwrap().contract_class();
    let (contract_address, _) = contract.deploy(@array![]).unwrap();

    let safe_dispatcher = IMyContractSafeDispatcher { contract_address };
    let res = safe_dispatcher.my_function();
    match res {
        Result::Ok(_) => panic!("Expected an error"),
        Result::Err(err_data) => {
            assert(*err_data.at(0) == 'ENTRYPOINT_NOT_FOUND', *err_data.at(0));
            assert(*err_data.at(1) == 'ENTRYPOINT_FAILED', *err_data.at(1));
        },
    };
}
