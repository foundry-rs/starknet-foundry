use core::clone::Clone;
use snforge_std::cheatcodes::contract_class::DeclareResultTrait;
use snforge_std::{declare, ContractClassTrait};
use starknet::{SyscallResultTrait};
use starknet::syscalls::deploy_syscall;

#[test]
fn test_deploy() {
    let empty_hash = declare("Empty").unwrap().contract_class().class_hash.clone();
    let proxy = declare("TraceInfoProxy").unwrap().contract_class().clone();
    let checker = declare("TraceInfoChecker").unwrap().contract_class();

    trace_resources::use_builtins_and_syscalls(empty_hash, 7);

    let (checker_address, _) = checker.deploy(@array![]).unwrap();

    proxy.deploy(@array![checker_address.into(), empty_hash.into(), 1]).unwrap();

    deploy_syscall(
        proxy.class_hash, 0, array![checker_address.into(), empty_hash.into(), 2].span(), false
    )
        .unwrap_syscall();

    proxy
        .deploy_at(@array![checker_address.into(), empty_hash.into(), 3], 123.try_into().unwrap())
        .unwrap();

    deploy_syscall(
        proxy.class_hash, 12412, array![checker_address.into(), empty_hash.into(), 4].span(), false
    )
        .unwrap_syscall();
}
