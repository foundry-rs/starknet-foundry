use snforge_std::{set_balance, Token, get_balance, CustomToken};
use core::starknet::contract_address::ContractAddress;

#[test]
fn test_cheat_strk() {
    let user_address: ContractAddress = 0x123.try_into().unwrap();

    let balance = get_balance(user_address, Token::STRK);
    assert(balance == 0, 'balance is not as expected');

    set_balance(user_address, 1_000_000, Token::STRK);

    let balance = get_balance(user_address, Token::STRK);
    assert(balance == 1_000_000, 'balance is not as expected');
}
