use starknet::SyscallResultTrait;
use traits::TryInto;
use starknet::ContractAddress;

use snforge_std::{
    Token, CustomToken, declare, ContractClassTrait, ContractClass, set_balance, DeclareResultTrait,
    map_entry_address, TokenTrait, TokenImpl,
};


use deal_tests::erc20::IERC20Dispatcher;
use deal_tests::erc20::IERC20DispatcherTrait;
use starknet::syscalls::{call_contract_syscall};

fn deploy_contract(name: ByteArray, constructor_calldata: Array<felt252>) -> ContractAddress {
    let contract = declare(name).unwrap().contract_class();
    let (contract_address, _) = contract.deploy(@constructor_calldata).unwrap();
    contract_address
}


#[test]
fn cheat_custom_token_balance() {
    // Deploy example ERC20 contract
    let token_contract_address = deploy_contract(
        "ERC20", array!['CustomToken ', 'CTKN', 18, 999_000_000, 0, 1234],
    );

    // Example contract address
    let user_address: ContractAddress = 0x123.try_into().unwrap();

    // Create custom token
    let token = Token::Custom(
        CustomToken {
            contract_address: token_contract_address,
            // in this example, we assume that the balances variable is called "balances"
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
