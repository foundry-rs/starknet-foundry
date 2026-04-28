use debugging_no_abi::{NestedDispatcher, NestedDispatcherTrait};
use snforge_std::cheatcodes::contract_class::DeclareResultTrait;
use snforge_std::{ContractClassTrait, declare};
use starknet::SyscallResultTrait;

#[test]
fn test_nested_safe_call_no_abi() {
    let (caller_contract, _) = declare("CallerContract")
        .unwrap()
        .contract_class()
        .deploy(@array![])
        .unwrap_syscall();

    let (failing_contract, _) = declare("FailingContract")
        .unwrap()
        .contract_class()
        .deploy(@array![])
        .unwrap();

    let dispatcher = NestedDispatcher { contract_address: caller_contract };
    dispatcher.nested(failing_contract);
}
