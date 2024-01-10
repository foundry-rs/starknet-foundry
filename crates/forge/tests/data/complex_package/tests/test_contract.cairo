use starknet::{ContractAddress, contract_address_const, get_execution_info_syscall};

use snforge_std::{
    declare, ContractClassTrait, start_prank, stop_prank, start_warp, CheatTarget, spy_events,
    SpyOn, EventSpy, EventAssertions, PrintTrait
};
use openzeppelin::token::erc20::{
    interface::{IERC20Dispatcher, IERC20DispatcherTrait}, erc20::ERC20Component
};
use complex_package::mock_erc20::{IERC20MintBurnDispatcher, IERC20MintBurnDispatcherTrait};
use complex_package::my_contract::{IMyContractDispatcher, IMyContractDispatcherTrait};

const INITIAL_BALANCE: u256 = 100_000000;
const TRANSFER_AMOUNT: u256 = 5_000000;

fn deploy_contract(name: felt252, calldata: @Array<felt252>) -> ContractAddress {
    let contract = declare(name);
    contract.deploy(calldata).unwrap()
}

fn alice() -> ContractAddress {
    return contract_address_const::<'alice'>();
}

fn setup() -> (IERC20Dispatcher, IMyContractDispatcher) {
    let token_address = deploy_contract('ERC20Mock', @array![6]);
    let token = IERC20Dispatcher { contract_address: token_address };

    let my_contract_address = deploy_contract('MyContract', @array![]);
    let my_contract = IMyContractDispatcher { contract_address: my_contract_address };

    return (token, my_contract);
}

#[test]
fn test_transfer() {
    let (token, my_contract) = setup();

    let dispatcher = IERC20MintBurnDispatcher { contract_address: token.contract_address };
    dispatcher.mint(alice(), INITIAL_BALANCE);

    let mut balance_alice = token.balance_of(alice());
    assert(balance_alice == INITIAL_BALANCE, 'Alice: initial balance');

    start_prank(CheatTarget::One(token.contract_address), alice());

    token.transfer(my_contract.contract_address, TRANSFER_AMOUNT);

    balance_alice = token.balance_of(alice());
    assert(balance_alice == INITIAL_BALANCE - TRANSFER_AMOUNT, 'Alice: transfer');

    let mut balance_contract = token.balance_of(my_contract.contract_address);
    assert(balance_contract == TRANSFER_AMOUNT, 'Contract: received');

    stop_prank(CheatTarget::One(token.contract_address));

    my_contract.claim(token, alice());

    balance_alice = token.balance_of(alice());
    balance_contract = token.balance_of(my_contract.contract_address);

    assert(balance_alice == INITIAL_BALANCE, 'Alice: claim');
    assert(balance_contract == 0, 'Contract: transfer');
}
