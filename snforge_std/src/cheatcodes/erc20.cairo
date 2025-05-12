use starknet::{ContractAddress};
use snforge_std::cheatcodes::storage::{map_entry_address, store};

const STRK_CONTRACT_ADDRESS: felt252 =
    0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d;
const STRK_BALANCES_VARIABLE_SELECTOR: felt252 =
    1648309034483306940318020057553480881073352647889682838905012914068126451082; // selector!("ERC20_balances");

#[derive(Drop, Serde, Copy, Debug)]
pub struct CustomToken {
    pub contract_address: ContractAddress,
    pub balances_variable_selector: felt252,
}

#[derive(Drop, Copy, Clone, Debug)]
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
            Token::STRK => STRK_BALANCES_VARIABLE_SELECTOR,
            Token::Custom(CustomToken {
                balances_variable_selector, ..,
            }) => balances_variable_selector,
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

    store(token.contract_address(), balance_low_address, array![new_balance.low.into()].span());
    store(token.contract_address(), balance_high_address, array![new_balance.high.into()].span());
}
