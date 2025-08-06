use snforge_std::{ContractClassTrait, DeclareResultTrait, declare};
use using_cheatcodes::{ICheatcodeCheckerDispatcher, ICheatcodeCheckerDispatcherTrait};

#[test]
fn call_and_invoke() {
    let contract = declare("CheatcodeChecker").unwrap().contract_class();
    let (contract_address, _) = contract.deploy(@array![]).unwrap();
    let dispatcher = ICheatcodeCheckerDispatcher { contract_address };

    let balance = dispatcher.get_balance();
    assert(balance == 0, 'balance == 0');

    dispatcher.increase_balance(100);

    let balance = dispatcher.get_balance();
    assert(balance == 100, 'balance == 100');
}
