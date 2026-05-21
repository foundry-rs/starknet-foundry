use debugging_no_abi::{ExternalDispatcher, ExternalDispatcherTrait};
use snforge_std::cheatcodes::contract_class::DeclareResultTrait;
use snforge_std::cheatcodes::execution_info::contract_address;
use snforge_std::{ContractClassTrait, declare};
use starknet::SyscallResultTrait;

#[test]
fn test_nested_safe_call_no_abi() {
    let (contract_address, _) = declare("CallerContract")
        .unwrap()
        .contract_class()
        .deploy(@array![])
        .unwrap_syscall();

    let dispatcher = ExternalDispatcher { contract_address };
    dispatcher.call();
}
