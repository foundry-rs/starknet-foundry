use array::ArrayTrait;
use result::ResultTrait;
use option::OptionTrait;
use traits::TryInto;
use starknet::ContractAddress;
use starknet::Felt252TryIntoContractAddress;
use cheatcodes::PreparedContract;

use dispatchers::erc20::IERC20Dispatcher;
use dispatchers::erc20::IERC20DispatcherTrait;
use dispatchers::hello_starknet::IHelloStarknetDispatcher;
use dispatchers::hello_starknet::IHelloStarknetDispatcherTrait;


#[test]
fn call_and_invoke() {
    let class_hash = declare('HelloStarknet').unwrap();
    let prepared = PreparedContract { contract_address: 1234, class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
    let contract_address = deploy(prepared).unwrap();
    let contract_address: ContractAddress = contract_address.try_into().unwrap();
    let dispatcher = IHelloStarknetDispatcher { contract_address };

    let balance = dispatcher.get_balance();
    assert(balance == 0, 'balance == 0');

    dispatcher.increase_balance(100);

    let balance = dispatcher.get_balance();
    assert(balance == 100, 'balance == 100');
}

#[test]
fn advanced_types() {
    let mut calldata = ArrayTrait::new();
    calldata.append('token');   // name
    calldata.append('TKN');     // symbol
    calldata.append(18);        // decimals
    calldata.append(1111);      // initial supply low
    calldata.append(0);         // initial supply high
    calldata.append(1234);      // recipient

    let class_hash = declare('ERC20').unwrap();
    let prepared = PreparedContract { contract_address: 4567, class_hash: class_hash, constructor_calldata: @calldata };
    let contract_address = deploy(prepared).unwrap();
    // let contract_address: ContractAddress = contract_address.try_into().unwrap();
    // let dispatcher = IERC20Dispatcher { contract_address };
    // let user_address: ContractAddress = 1234.try_into().unwrap();
    // let other_user_address: ContractAddress = 9999.try_into().unwrap();

    // let balance = dispatcher.balance_of(user_address);
    // assert(balance == 1111_u256, 'balance == 1111');

    // // TODO(#1986): Change that when we support mocking addresses, so we can actually call transfer
    // // dispatcher.transfer(other_user_address, 1000_u256);

    // // let balance = dispatcher.balance_of(user_address);
    // // assert(balance == 111_u256, 'balance == 111');
    // let balance = dispatcher.balance_of(other_user_address);
    // assert(balance == 0_u256, 'balance == 1000');
}
