use openzeppelin_token::erc20::{ERC20ABIDispatcher, ERC20ABIDispatcherTrait};
use snforge_std::{
    CheatSpan, ContractClassTrait, DeclareResultTrait, EventSpyAssertionsTrait,
    cheat_caller_address, declare, spy_events,
};
use starknet::{ContractAddress, contract_address_const};
use {{ PROJECT_NAME }}::token_sender::{
    ITokenSenderDispatcher, ITokenSenderDispatcherTrait, TokenSender, TransferRequest,
};

const INITIAL_SUPPLY: u256 = 10_000_000_000;

fn setup() -> (ContractAddress, ContractAddress) {
    let erc20_class_hash = declare("MockERC20").unwrap().contract_class();

    let mut calldata = ArrayTrait::new();
    INITIAL_SUPPLY.serialize(ref calldata);

    let sender_account = contract_address_const::<1>();
    sender_account.serialize(ref calldata);

    let (erc20_address, _) = erc20_class_hash.deploy(@calldata).unwrap();

    let token_sender_class_hash = declare("TokenSender").unwrap().contract_class();

    let mut calldata = ArrayTrait::new();

    let (token_sender_address, _) = token_sender_class_hash.deploy(@calldata).unwrap();

    (erc20_address, token_sender_address)
}

#[test]
fn test_single_send() {
    let (erc20_address, token_sender_address) = setup();
    let erc20 = ERC20ABIDispatcher { contract_address: erc20_address };

    let sender_account = contract_address_const::<1>();
    let target_account = contract_address_const::<2>();

    assert!(erc20.balance_of(sender_account) == INITIAL_SUPPLY, "Balance should be > 0");

    cheat_caller_address(erc20_address, sender_account, CheatSpan::TargetCalls(1));

    let transfer_value: u256 = 100;
    erc20.approve(token_sender_address, transfer_value * 2);

    assert!(
        erc20.allowance(sender_account, token_sender_address) == transfer_value * 2,
        "Allowance not set",
    );

    let token_sender = ITokenSenderDispatcher { contract_address: token_sender_address };
    let request = TransferRequest { recipient: target_account, amount: transfer_value };

    let mut transfer_list = ArrayTrait::<TransferRequest>::new();
    transfer_list.append(request);

    cheat_caller_address(token_sender_address, sender_account, CheatSpan::TargetCalls(1));
    token_sender.multisend(erc20_address, transfer_list);

    let balance_after = erc20.balance_of(target_account);
    assert!(balance_after == transfer_value, "Balance should be > 0");
}

#[test]
#[fuzzer]
fn test_single_send_fuzz(transfer_value: u256) {
    let (erc20_address, token_sender_address) = setup();
    let erc20 = ERC20ABIDispatcher { contract_address: erc20_address };

    let sender_account = contract_address_const::<1>();
    let target_account_1 = contract_address_const::<2>();

    assert!(erc20.balance_of(sender_account) == INITIAL_SUPPLY, "Balance should be > 0");

    cheat_caller_address(erc20_address, sender_account, CheatSpan::TargetCalls(1));

    let transfer_value: u256 = 100;
    erc20.approve(token_sender_address, transfer_value * 2);

    assert!(
        erc20.allowance(sender_account, token_sender_address) == transfer_value * 2,
        "Allowance not set",
    );

    let token_sender = ITokenSenderDispatcher { contract_address: token_sender_address };
    let request = TransferRequest { recipient: target_account_1, amount: transfer_value };

    let mut transfer_list = ArrayTrait::<TransferRequest>::new();
    transfer_list.append(request);

    let mut spy = spy_events();

    cheat_caller_address(token_sender_address, sender_account, CheatSpan::TargetCalls(1));
    token_sender.multisend(erc20_address, transfer_list);

    spy
        .assert_emitted(
            @array![
                (
                    token_sender_address,
                    TokenSender::Event::TransferSent(
                        TokenSender::TransferSent {
                            recipient: target_account_1,
                            token_address: erc20_address,
                            amount: transfer_value,
                        },
                    ),
                ),
            ],
        );

    let balance_after = erc20.balance_of(target_account_1);
    assert!(balance_after == transfer_value, "Balance should be > 0");
}

#[test]
fn test_multisend() {
    let (erc20_address, token_sender_address) = setup();
    let erc20 = ERC20ABIDispatcher { contract_address: erc20_address };

    let sender_account = contract_address_const::<1>();
    let target_account_1 = contract_address_const::<2>();
    let target_account_2 = contract_address_const::<3>();

    assert!(erc20.balance_of(sender_account) == INITIAL_SUPPLY, "Balance should be > 0");

    cheat_caller_address(erc20_address, sender_account, CheatSpan::TargetCalls(1));

    let transfer_value: u256 = 100;
    erc20.approve(token_sender_address, transfer_value * 2);

    assert!(
        erc20.allowance(sender_account, token_sender_address) == transfer_value * 2,
        "Allowance not set",
    );

    let token_sender = ITokenSenderDispatcher { contract_address: token_sender_address };
    let request_1 = TransferRequest { recipient: target_account_1, amount: transfer_value };
    let request_2 = TransferRequest { recipient: target_account_2, amount: transfer_value };

    let mut transfer_list = ArrayTrait::<TransferRequest>::new();
    transfer_list.append(request_1);
    transfer_list.append(request_2);

    cheat_caller_address(token_sender_address, sender_account, CheatSpan::TargetCalls(1));
    token_sender.multisend(erc20_address, transfer_list);

    let balance_after = erc20.balance_of(target_account_1);
    assert!(balance_after == transfer_value, "Balance should be > 0");

    let balance_after = erc20.balance_of(target_account_2);
    assert!(balance_after == transfer_value, "Balance should be > 0");
}
