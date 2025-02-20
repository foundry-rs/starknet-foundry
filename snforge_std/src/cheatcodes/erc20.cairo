use starknet::{ContractAddress};
use snforge_std::cheatcodes::storage::{map_entry_address, store};

pub const STRK_CONTRACT_ADDRESS: felt252 =
    0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d;

#[derive(Drop, Serde, Copy)]
pub struct CustomToken {
    pub contract_address: ContractAddress,
    pub balances_variable_selector: felt252,
}

#[derive(Drop, Copy, Clone)]
pub enum Token {
    STRK,
    Custom: CustomToken,
}

#[generate_trait]
pub impl TokenImpl of TokenTrait {
    fn contract_address(self: Token) -> ContractAddress {
        match self {
            Token::STRK => STRK_CONTRACT_ADDRESS.try_into().unwrap(),
            Token::Custom(CustomToken { contract_address, .. }) => contract_address,
        }
    }

    fn balances_variable_selector(self: Token) -> felt252 {
        match self {
            Token::STRK => selector!("ERC20_balances"),
            Token::Custom(CustomToken { balances_variable_selector,
            .., }) => balances_variable_selector,
        }
    }
}

/// Sets the balance of `token`` for `target`` contract to `new_balance`
/// - `target` - address of the contract, which balance you want to modify
/// - `new_balance` - new balance value
/// - `token` - token for which the balance is being set
pub fn set_balance(target: ContractAddress, new_balance: u256, token: Token) {
    let balance_low_address = map_entry_address(
        token.balances_variable_selector(), [target.into()].span(),
    );
    let balance_high_address = balance_low_address + 1;
    // let balance_high_address = map_entry_address(
    //     token.balances_variable_selector(), [target.into(), 1].span(),
    // );

    store(token.contract_address(), balance_high_address, array![new_balance.low.into()].span());
    store(token.contract_address(), balance_low_address, array![new_balance.high.into()].span());
}
