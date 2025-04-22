use starknet::SyscallResultTrait;
use starknet::ContractAddress;

use snforge_std::{
    Token, CustomToken, declare, ContractClassTrait, set_balance, DeclareResultTrait, TokenTrait,
    TokenImpl,
};


use starknet::syscalls::{call_contract_syscall};

fn deploy_contract(name: ByteArray, constructor_calldata: Array<felt252>) -> ContractAddress {
    let contract = declare(name).unwrap().contract_class();
    let (contract_address, _) = contract.deploy(@constructor_calldata).unwrap();
    contract_address
}


#[test]
fn set_balance_custom_token() {
    // Deploy your own token with ERC20 contract
    let constructor_calldata = array![
        'CustomToken ', // Token name
        'CTKN', // Token symbol
        18, // Decimals
        999_000_000, // Initial supply (u256 high)
        0, // Initial supply (u256 low)
        1234 // Recipient address
    ];
    let token_contract_address = deploy_contract("ERC20", constructor_calldata);

    // Example user address, whose balance we want to set
    let user_address: ContractAddress = 0x123.try_into().unwrap();

    // Create custom token
    let token = Token::Custom(
        CustomToken {
            contract_address: token_contract_address,
            // In this example, we assume that the balances variable is named `balances`
            balances_variable_selector: selector!("balances"),
        },
    );

    set_balance(user_address, 1_000_000, token);

    // Read the balance
    let balance = call_contract_syscall(
        token.contract_address().into(),
        selector!("balance_of"),
        array![user_address.into()].span(),
    )
        .unwrap_syscall();

    assert(balance == array![0, 1_000_000].span(), 'Invalid balance');
}
