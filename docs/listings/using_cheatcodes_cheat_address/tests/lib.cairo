use snforge_std::{declare, ContractClassTrait, DeclareResultTrait, start_cheat_caller_address};
use using_cheatcodes_cheat_address::{ICheatcodeCheckerDispatcher, ICheatcodeCheckerDispatcherTrait};

#[test]
fn call_and_invoke() {
    let contract = declare("CheatcodeChecker").unwrap().contract_class();
    let (contract_address, _) = contract.deploy(@array![]).unwrap();
    let dispatcher = ICheatcodeCheckerDispatcher { contract_address };

    let balance = dispatcher.get_balance();
    assert(balance == 0, 'balance == 0');

    // Change the caller address to 123 when calling the contract at the `contract_address` address
    start_cheat_caller_address(contract_address, 123.try_into().unwrap());

    dispatcher.increase_balance(100);

    let balance = dispatcher.get_balance();
    assert(balance == 100, 'balance == 100');
}
