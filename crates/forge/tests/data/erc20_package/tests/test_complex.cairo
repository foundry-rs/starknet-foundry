use erc20_package::erc20::{IERC20Dispatcher, IERC20DispatcherTrait};
use snforge_std::cheatcodes::contract_class::DeclareResultTrait;
use snforge_std::{
    ContractClassTrait, declare, start_cheat_caller_address, stop_cheat_caller_address,
    test_address,
};
use starknet::ContractAddress;

const NAME: felt252 = 'TOKEN';
const SYMBOL: felt252 = 'TKN';
const DECIMALS: u8 = 2;
const INITIAL_SUPPLY: u256 = 10;

fn deploy_erc20(
    name: felt252, symbol: felt252, decimals: u8, initial_supply: u256, recipient: ContractAddress,
) -> ContractAddress {
    let contract = declare("ERC20").unwrap().contract_class();

    let mut constructor_calldata: Array<felt252> = array![name, symbol, decimals.into()];

    let mut initial_supply_serialized = array![];
    initial_supply.serialize(ref initial_supply_serialized);

    constructor_calldata.append_span(initial_supply_serialized.span());
    constructor_calldata.append(recipient.into());

    let (address, _) = contract.deploy(@constructor_calldata).unwrap();
    address
}

// Syscalls from constructor are not counted
// StorageRead: 22, StorageWrite: 12, EmitEvent: 4, GetExecutionInfo: 3
#[test]
fn complex() {
    let erc20_address = deploy_erc20(NAME, SYMBOL, DECIMALS, INITIAL_SUPPLY, test_address());
    let dispatcher = IERC20Dispatcher { contract_address: erc20_address };

    let spender: ContractAddress = 123.try_into().unwrap();

    // GetExecutionInfo: 1, StorageRead: 4, StorageWrite: 4, EmitEvent: 1
    dispatcher.transfer(spender, 2.into());

    // StorageRead: 2
    let spender_balance = dispatcher.balance_of(spender);
    assert(spender_balance == 2, 'invalid spender balance');

    start_cheat_caller_address(erc20_address, spender);

    // GetExecutionInfo: 1, StorageRead: 2, StorageWrite: 2, EmitEvent: 1
    dispatcher.increase_allowance(test_address(), 2);

    // StorageRead: 2
    let allowance = dispatcher.allowance(spender, test_address());
    assert(allowance == 2, 'invalid allowance');

    stop_cheat_caller_address(erc20_address);

    // GetExecutionInfo: 1, StorageRead: 6, StorageWrite: 6, EmitEvent: 2
    dispatcher.transfer_from(spender, test_address(), 2);

    // StorageRead: 2
    let allowance = dispatcher.allowance(spender, test_address());
    assert(allowance == 0, 'invalid allowance');

    // StorageRead: 2
    let spender_balance = dispatcher.balance_of(spender);
    assert(spender_balance == 0, 'invalid spender balance');

    // StorageRead: 2
    let balance = dispatcher.balance_of(test_address());
    assert(balance == INITIAL_SUPPLY, 'invalid balance');
}
