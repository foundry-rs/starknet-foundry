use snforge_std::{CheatSpan, ContractClassTrait, DeclareResultTrait, cheat_caller_address, declare};
use starknet::ContractAddress;
use using_cheatcodes_others::{
    ICheatcodeCheckerSafeDispatcher, ICheatcodeCheckerSafeDispatcherTrait,
};

#[test]
#[feature("safe_dispatcher")]
fn call_and_invoke() {
    let contract = declare("CheatcodeChecker").unwrap().contract_class();
    let (contract_address, _) = contract.deploy(@array![]).unwrap();
    let safe_dispatcher = ICheatcodeCheckerSafeDispatcher { contract_address };

    let balance = safe_dispatcher.get_balance().unwrap();
    assert_eq!(balance, 0);

    // Function `increase_balance` from HelloStarknet contract
    // requires the caller_address to be 123
    let spoofed_caller: ContractAddress = 123.try_into().unwrap();

    // Change the caller address for the contract_address for a span of 2 target calls (here, calls
    // to contract_address)
    cheat_caller_address(contract_address, spoofed_caller, CheatSpan::TargetCalls(2));

    // Call #1 should succeed
    let call_1_result = safe_dispatcher.increase_balance(100);
    assert!(call_1_result.is_ok());

    // Call #2 should succeed
    let call_2_result = safe_dispatcher.increase_balance(100);
    assert!(call_2_result.is_ok());

    // Call #3 should fail, as the cheat_caller_address cheatcode has been canceled
    let call_3_result = safe_dispatcher.increase_balance(100);
    assert_eq!(call_3_result, Result::Err(array!['user is not allowed']));

    let balance = safe_dispatcher.get_balance().unwrap();
    assert_eq!(balance, 200);
}
