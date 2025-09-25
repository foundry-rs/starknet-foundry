use core::clone::Clone;
use snforge_std::cheatcodes::contract_class::DeclareResultTrait;
use snforge_std::{ContractClassTrait, L1HandlerTrait, declare};

#[test]
fn test_l1_handler() {
    let empty_hash = declare("Empty").unwrap().contract_class().class_hash.clone();
    let proxy = declare("TraceInfoProxy").unwrap().contract_class();
    let checker = declare("TraceInfoChecker").unwrap().contract_class();

    trace_resources::use_builtins_and_syscalls(empty_hash, 7);

    let (checker_address, _) = checker.deploy(@array![]).unwrap();
    let (proxy_address, _) = proxy
        .deploy(@array![checker_address.into(), empty_hash.into(), 1])
        .unwrap();

    let mut l1_handler = L1HandlerTrait::new(checker_address, selector!("handle_l1_message"));

    l1_handler.execute(123, array![proxy_address.into(), empty_hash.into(), 2].span()).unwrap();
}
