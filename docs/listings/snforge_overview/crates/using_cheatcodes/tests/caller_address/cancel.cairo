//ANCHOR:first_half
use snforge_std::{
    declare, ContractClassTrait, DeclareResultTrait, start_cheat_caller_address,
    stop_cheat_caller_address
};

use using_cheatcodes::{ICheatcodeCheckerSafeDispatcher, ICheatcodeCheckerSafeDispatcherTrait};

#[test]
//ANCHOR_END:first_half
#[should_panic(expected: 'Second call failed!')]
//ANCHOR:second_half
#[feature("safe_dispatcher")]
fn call_and_invoke() {
    let contract = declare("CheatcodeChecker").unwrap().contract_class();
    let (contract_address, _) = contract.deploy(@array![]).unwrap();
    let dispatcher = ICheatcodeCheckerSafeDispatcher { contract_address };

    let balance = dispatcher.get_balance().unwrap();
    assert(balance == 0, 'balance == 0');

    // Change the caller address to 123 when calling the contract at the `contract_address` address
    start_cheat_caller_address(contract_address, 123.try_into().unwrap());

    // Call to method with caller restriction succeeds
    dispatcher.increase_balance(100).expect('First call failed!');

    let balance = dispatcher.get_balance();
    assert_eq!(balance, Result::Ok(100));

    // Cancel the cheat
    stop_cheat_caller_address(contract_address);

    // The call fails now
    dispatcher.increase_balance(100).expect('Second call failed!');

    let balance = dispatcher.get_balance();
    assert_eq!(balance, Result::Ok(100));
}
//ANCHOR_END:second_half

mod dummy {} // trick `scarb fmt -c`
