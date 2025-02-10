use snforge_std::{set_balance, Token, get_balance, CustomToken};
use core::starknet::contract_address::ContractAddress;

#[test]
fn test_cheat_custom_token() {
    let user_address: ContractAddress = 0x123.try_into().unwrap();

    // Define a custom token with address 0x456
    // let's assume it's balances storage variable name is 'custom_ERC20_balances'
    let custom_token = CustomToken {
        address: 0x456.try_into().unwrap(),
        balances_storage_address: selector!("custom_ERC20_balances")
    };

    let balance = get_balance(user_address, Token::Custom(custom_token));
    assert(balance == 0, 'balance is not as expected');

    set_balance(user_address, 1_000_000, Token::Custom(custom_token.clone()));

    let balance = get_balance(user_address, Token::Custom(custom_token.clone()));
    assert(balance == 1_000_000, 'balance is not as expected');
}
