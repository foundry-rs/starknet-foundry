use openzeppelin_token::erc20::{ERC20ABIDispatcher, ERC20ABIDispatcherTrait, ERC20Component};
use snforge_std::{
    CheatSpan, ContractClassTrait, DeclareResultTrait, EventSpyAssertionsTrait,
    cheat_caller_address, declare, spy_events,
};
use starknet::{ContractAddress, contract_address_const};

const STRK_TOKEN_ADDRESS: felt252 =
    0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d;

const INITIAL_SUPPLY: u256 = 10_000_000_000;

fn setup() -> ContractAddress {
    let erc20_class_hash = declare("MockERC20").unwrap().contract_class();

    let mut calldata = ArrayTrait::new();
    INITIAL_SUPPLY.serialize(ref calldata);

    let sender_account = contract_address_const::<1>();
    sender_account.serialize(ref calldata);

    let (contract_address, _) = erc20_class_hash.deploy(@calldata).unwrap();

    contract_address
}

#[test]
fn test_get_balance() {
    let contract_address = setup();
    let erc20 = ERC20ABIDispatcher { contract_address };

    let sender_account = contract_address_const::<1>();

    assert!(erc20.balance_of(sender_account) == INITIAL_SUPPLY, "Balance should be > 0");
}

#[test]
fn test_transfer() {
    let contract_address = setup();
    let erc20 = ERC20ABIDispatcher { contract_address };

    let sender_account = contract_address_const::<1>();
    let target_account = contract_address_const::<2>();

    let balance_before = erc20.balance_of(target_account);
    assert!(balance_before == 0, "Invalid balance");

    cheat_caller_address(contract_address, sender_account, CheatSpan::TargetCalls(1));

    let transfer_value: u256 = 100;
    erc20.transfer(target_account, transfer_value);

    let balance_after = erc20.balance_of(target_account);
    assert!(balance_after == transfer_value, "No value transferred");
}

#[test]
#[fork("SEPOLIA_LATEST", block_number: 61804)]
fn test_fork_transfer() {
    let target_account = contract_address_const::<2>();
    let strk_contract_address = contract_address_const::<STRK_TOKEN_ADDRESS>();

    let erc20 = ERC20ABIDispatcher { contract_address: strk_contract_address };

    let owner_account: ContractAddress =
        0x04337e199aa6a8959aeb2a6afcd2f82609211104191a041e7b9ba2f4039768f0
        .try_into()
        .unwrap();

    let balance_before = erc20.balance_of(target_account);
    assert!(balance_before == 0, "Invalid balance");

    cheat_caller_address(strk_contract_address, owner_account, CheatSpan::TargetCalls(1));

    let transfer_value: u256 = 100;
    erc20.transfer(target_account, transfer_value);

    let balance_after = erc20.balance_of(target_account);
    assert!(balance_after == transfer_value, "No value transferred");
}

#[test]
fn test_transfer_event() {
    let contract_address = setup();
    let erc20 = ERC20ABIDispatcher { contract_address };

    let sender_account = contract_address_const::<1>();
    let target_account = contract_address_const::<2>();

    cheat_caller_address(contract_address, sender_account, CheatSpan::TargetCalls(1));

    let mut spy = spy_events();

    let transfer_value: u256 = 100;
    erc20.transfer(target_account, transfer_value);

    spy
        .assert_emitted(
            @array![
                (
                    contract_address,
                    ERC20Component::Event::Transfer(
                        ERC20Component::Transfer {
                            from: sender_account, to: target_account, value: transfer_value,
                        },
                    ),
                ),
            ],
        );
}

#[test]
#[should_panic(expected: ('ERC20: insufficient balance',))]
fn should_panic_transfer() {
    let contract_address = setup();
    let erc20 = ERC20ABIDispatcher { contract_address };

    let sender_account = contract_address_const::<1>();
    let target_account = contract_address_const::<2>();

    let balance_before = erc20.balance_of(target_account);
    assert!(balance_before == 0, "Invalid balance");

    cheat_caller_address(contract_address, sender_account, CheatSpan::TargetCalls(1));

    let transfer_value: u256 = INITIAL_SUPPLY + 1;

    erc20.transfer(target_account, transfer_value);
}
