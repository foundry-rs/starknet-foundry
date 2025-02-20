use starknet::SyscallResultTrait;
use traits::TryInto;
use starknet::ContractAddress;

use snforge_std::{Token, CustomToken, set_balance, TokenTrait, TokenImpl};

use deal_tests::erc20::IERC20Dispatcher;
use deal_tests::erc20::IERC20DispatcherTrait;
use starknet::syscalls::call_contract_syscall;

#[test]
fn cheat_strk_balance() {
    // Example contract address
    let user_address: ContractAddress = 0x123.try_into().unwrap();
    let token = Token::STRK;

    set_balance(user_address, 1_000_000, token);

    // Read the balance
    let balance = call_contract_syscall(
        // We can read token address by using `contract_address()` method
        token.contract_address().into(),
        selector!("balance_of"),
        array![user_address.into()].span(),
    )
        .unwrap_syscall();

    assert(balance == array![0, 1_000_000].span(), 'Invalid balance');
}
