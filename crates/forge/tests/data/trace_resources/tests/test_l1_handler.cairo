use snforge_std::{declare, ContractClassTrait, L1HandlerTrait};

#[test]
fn test_l1_handler() {
    let empty_hash = declare('Empty').class_hash;
    let proxy = declare('TraceInfoProxy');
    let checker = declare('TraceInfoChecker');

    trace_resources::use_builtins_and_syscalls(empty_hash, 7);

    let checker_address = checker.deploy(@array![]).unwrap();
    let proxy_address = proxy
        .deploy(@array![checker_address.into(), empty_hash.into(), 1])
        .unwrap();

    let mut l1_handler = L1HandlerTrait::new(checker_address, function_name: 'handle_l1_message');

    l1_handler.from_address = 123;
    l1_handler.payload = array![proxy_address.into(), empty_hash.into(), 2].span();

    l1_handler.execute().unwrap();
}
