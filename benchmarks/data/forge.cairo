use array::ArrayTrait;
use result::ResultTrait;
use option::OptionTrait;
use traits::TryInto;
use starknet::ContractAddress;
use starknet::Felt252TryIntoContractAddress;
use cheatcodes::PreparedContract;

use project::IHelloStarknetSafeDispatcher;
use project::IHelloStarknetSafeDispatcherTrait;

fn deploy_hello_starknet() -> ContractAddress {
    let class_hash = declare('HelloStarknet').unwrap();
    let prepared = PreparedContract {
        contract_address: 1234, class_hash: class_hash, constructor_calldata: @ArrayTrait::new()
    };
    let contract_address = deploy(prepared).unwrap();

    let contract_address: ContractAddress = contract_address.try_into().unwrap();

    contract_address
}

#[test]
fn test_increase_balance_1() {
    let contract_address = deploy_hello_starknet();

    let safe_dispatcher = IHelloStarknetSafeDispatcher { contract_address };

    let balance_before = safe_dispatcher.get_balance().unwrap();
    assert(balance_before == 0, 'Invalid balance');

    safe_dispatcher.increase_balance(42).unwrap();

    let balance_after = safe_dispatcher.get_balance().unwrap();
    assert(balance_after == 42, 'Invalid balance');
}

#[test]
fn test_increase_balance_2() {
    let contract_address = deploy_hello_starknet();

    let safe_dispatcher = IHelloStarknetSafeDispatcher { contract_address };

    let balance_before = safe_dispatcher.get_balance().unwrap();
    assert(balance_before == 0, 'Invalid balance');

    safe_dispatcher.increase_balance(42).unwrap();

    let balance_after = safe_dispatcher.get_balance().unwrap();
    assert(balance_after == 42, 'Invalid balance');
}

#[test]
fn test_increase_balance_3() {
    let contract_address = deploy_hello_starknet();

    let safe_dispatcher = IHelloStarknetSafeDispatcher { contract_address };

    let balance_before = safe_dispatcher.get_balance().unwrap();
    assert(balance_before == 0, 'Invalid balance');

    safe_dispatcher.increase_balance(42).unwrap();

    let balance_after = safe_dispatcher.get_balance().unwrap();
    assert(balance_after == 42, 'Invalid balance');
}

#[test]
fn test_increase_balance_4() {
    let contract_address = deploy_hello_starknet();

    let safe_dispatcher = IHelloStarknetSafeDispatcher { contract_address };

    let balance_before = safe_dispatcher.get_balance().unwrap();
    assert(balance_before == 0, 'Invalid balance');

    safe_dispatcher.increase_balance(42).unwrap();

    let balance_after = safe_dispatcher.get_balance().unwrap();
    assert(balance_after == 42, 'Invalid balance');
}

#[test]
fn test_increase_balance_5() {
    let contract_address = deploy_hello_starknet();

    let safe_dispatcher = IHelloStarknetSafeDispatcher { contract_address };

    let balance_before = safe_dispatcher.get_balance().unwrap();
    assert(balance_before == 0, 'Invalid balance');

    safe_dispatcher.increase_balance(42).unwrap();

    let balance_after = safe_dispatcher.get_balance().unwrap();
    assert(balance_after == 42, 'Invalid balance');
}

#[test]
fn test_increase_balance_6() {
    let contract_address = deploy_hello_starknet();

    let safe_dispatcher = IHelloStarknetSafeDispatcher { contract_address };

    let balance_before = safe_dispatcher.get_balance().unwrap();
    assert(balance_before == 0, 'Invalid balance');

    safe_dispatcher.increase_balance(42).unwrap();

    let balance_after = safe_dispatcher.get_balance().unwrap();
    assert(balance_after == 42, 'Invalid balance');
}

#[test]
fn test_increase_balance_7() {
    let contract_address = deploy_hello_starknet();

    let safe_dispatcher = IHelloStarknetSafeDispatcher { contract_address };

    let balance_before = safe_dispatcher.get_balance().unwrap();
    assert(balance_before == 0, 'Invalid balance');

    safe_dispatcher.increase_balance(42).unwrap();

    let balance_after = safe_dispatcher.get_balance().unwrap();
    assert(balance_after == 42, 'Invalid balance');
}

#[test]
fn test_increase_balance_8() {
    let contract_address = deploy_hello_starknet();

    let safe_dispatcher = IHelloStarknetSafeDispatcher { contract_address };

    let balance_before = safe_dispatcher.get_balance().unwrap();
    assert(balance_before == 0, 'Invalid balance');

    safe_dispatcher.increase_balance(42).unwrap();

    let balance_after = safe_dispatcher.get_balance().unwrap();
    assert(balance_after == 42, 'Invalid balance');
}

#[test]
fn test_increase_balance_9() {
    let contract_address = deploy_hello_starknet();

    let safe_dispatcher = IHelloStarknetSafeDispatcher { contract_address };

    let balance_before = safe_dispatcher.get_balance().unwrap();
    assert(balance_before == 0, 'Invalid balance');

    safe_dispatcher.increase_balance(42).unwrap();

    let balance_after = safe_dispatcher.get_balance().unwrap();
    assert(balance_after == 42, 'Invalid balance');
}

#[test]
fn test_increase_balance_10() {
    let contract_address = deploy_hello_starknet();

    let safe_dispatcher = IHelloStarknetSafeDispatcher { contract_address };

    let balance_before = safe_dispatcher.get_balance().unwrap();
    assert(balance_before == 0, 'Invalid balance');

    safe_dispatcher.increase_balance(42).unwrap();

    let balance_after = safe_dispatcher.get_balance().unwrap();
    assert(balance_after == 42, 'Invalid balance');
}

#[test]
fn test_increase_balance_11() {
    let contract_address = deploy_hello_starknet();

    let safe_dispatcher = IHelloStarknetSafeDispatcher { contract_address };

    let balance_before = safe_dispatcher.get_balance().unwrap();
    assert(balance_before == 0, 'Invalid balance');

    safe_dispatcher.increase_balance(42).unwrap();

    let balance_after = safe_dispatcher.get_balance().unwrap();
    assert(balance_after == 42, 'Invalid balance');
}

#[test]
fn test_increase_balance_12() {
    let contract_address = deploy_hello_starknet();

    let safe_dispatcher = IHelloStarknetSafeDispatcher { contract_address };

    let balance_before = safe_dispatcher.get_balance().unwrap();
    assert(balance_before == 0, 'Invalid balance');

    safe_dispatcher.increase_balance(42).unwrap();

    let balance_after = safe_dispatcher.get_balance().unwrap();
    assert(balance_after == 42, 'Invalid balance');
}

#[test]
fn test_increase_balance_13() {
    let contract_address = deploy_hello_starknet();

    let safe_dispatcher = IHelloStarknetSafeDispatcher { contract_address };

    let balance_before = safe_dispatcher.get_balance().unwrap();
    assert(balance_before == 0, 'Invalid balance');

    safe_dispatcher.increase_balance(42).unwrap();

    let balance_after = safe_dispatcher.get_balance().unwrap();
    assert(balance_after == 42, 'Invalid balance');
}

#[test]
fn test_increase_balance_14() {
    let contract_address = deploy_hello_starknet();

    let safe_dispatcher = IHelloStarknetSafeDispatcher { contract_address };

    let balance_before = safe_dispatcher.get_balance().unwrap();
    assert(balance_before == 0, 'Invalid balance');

    safe_dispatcher.increase_balance(42).unwrap();

    let balance_after = safe_dispatcher.get_balance().unwrap();
    assert(balance_after == 42, 'Invalid balance');
}

#[test]
fn test_increase_balance_15() {
    let contract_address = deploy_hello_starknet();

    let safe_dispatcher = IHelloStarknetSafeDispatcher { contract_address };

    let balance_before = safe_dispatcher.get_balance().unwrap();
    assert(balance_before == 0, 'Invalid balance');

    safe_dispatcher.increase_balance(42).unwrap();

    let balance_after = safe_dispatcher.get_balance().unwrap();
    assert(balance_after == 42, 'Invalid balance');
}
