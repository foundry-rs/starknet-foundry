use snforge_std::{
    ContractClassTrait, DeclareResultTrait, declare, start_cheat_caller_address_global,
    stop_cheat_caller_address_global,
};
use using_cheatcodes_others::{ICheatcodeCheckerDispatcher, ICheatcodeCheckerDispatcherTrait};

#[test]
fn call_and_invoke_global() {
    let contract = declare("CheatcodeChecker").unwrap().contract_class();
    let (contract_address_a, _) = contract.deploy(@array![]).unwrap();
    let (contract_address_b, _) = contract.deploy(@array![]).unwrap();
    let dispatcher_a = ICheatcodeCheckerDispatcher { contract_address: contract_address_a };
    let dispatcher_b = ICheatcodeCheckerDispatcher { contract_address: contract_address_b };

    let balance_a = dispatcher_a.get_balance();
    let balance_b = dispatcher_b.get_balance();
    assert_eq!(balance_a, 0);
    assert_eq!(balance_b, 0);

    // Change the caller address to 123, both targets a and b will be affected
    // global cheatcodes work indefinitely until stopped
    start_cheat_caller_address_global(123.try_into().unwrap());

    dispatcher_a.increase_balance(100);
    dispatcher_b.increase_balance(100);

    let balance_a = dispatcher_a.get_balance();
    let balance_b = dispatcher_b.get_balance();
    assert_eq!(balance_a, 100);
    assert_eq!(balance_b, 100);

    // Cancel the cheat
    stop_cheat_caller_address_global();
}
