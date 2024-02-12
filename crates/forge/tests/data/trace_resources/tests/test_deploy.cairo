use snforge_std::{declare, ContractClassTrait};
use starknet::{SyscallResultTrait, deploy_syscall};

#[test]
fn test_deploy() {
    let empty_hash = declare('Empty').class_hash;
    let proxy = declare('TraceInfoProxy');
    let checker = declare('TraceInfoChecker');

    trace_resources::use_builtins_and_syscalls(empty_hash, 7);

    let checker_address = checker.deploy(@array![]).unwrap();

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
