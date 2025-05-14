use snforge_std::{Token, TokenImpl, TokenTrait, set_balance};
use starknet::syscalls::call_contract_syscall;
use starknet::{ContractAddress, SyscallResultTrait};

#[test]
fn set_balance_strk() {
    // Example user address, whose balance we want to set
    let user_address: ContractAddress = 0x123.try_into().unwrap();

    set_balance(user_address, 1_000_000, Token::STRK);

    // Read the balance
    let balance = call_contract_syscall(
        Token::STRK.contract_address().into(),
        selector!("balance_of"),
        array![user_address.into()].span(),
    )
        .unwrap_syscall();

    assert(balance == array![1_000_000, 0].span(), 'Invalid balance');
}

