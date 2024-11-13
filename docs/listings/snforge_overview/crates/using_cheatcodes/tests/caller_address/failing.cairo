//ANCHOR:first_half
use snforge_std::{declare, ContractClassTrait, DeclareResultTrait};
use using_cheatcodes::{ICheatcodeCheckerDispatcher, ICheatcodeCheckerDispatcherTrait};

#[test]
//ANCHOR_END:first_half
#[should_panic(expected: 'user is not allowed')]
//ANCHOR:second_half
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
//ANCHOR_END:second_half

mod dummy {} // trick `scarb fmt -c`
