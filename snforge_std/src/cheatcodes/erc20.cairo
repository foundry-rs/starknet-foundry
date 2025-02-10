use starknet::{ContractAddress, StorageAddress};
use snforge_std::cheatcodes::storage::{map_entry_address, store, load};

pub const STRK_CONTRACT_ADDRESS: felt252 =
    0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d;
pub const ETH_CONTRACT_ADDRESS: felt252 =
    0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d;

#[derive(Drop, Serde, Copy)]
pub struct CustomToken {
    pub address: ContractAddress,
    pub balances_storage_address: felt252,
}

#[derive(Drop)]
pub enum Token {
    STRK,
    ETH,
    Custom: CustomToken,
}

/// Stores felts from `serialized_value` in `target` contract's storage, starting at
/// `storage_address`.
/// - `target` - address of the contract, which storage you want to modify
/// - `storage_address` - offset of the data in the contract's storage
/// - `serialized_value` - a sequence of felts that will be inserted starting at `storage_address`
///
///

/// Sets the balance of `token`` for `target`` contract to `new_balance`
/// - `target` - address of the contract, which balance you want to modify
/// - `new_balance` - new balance value
/// - `token` - token for which the balance is being set
pub fn set_balance(target: ContractAddress, new_balance: u256, token: Token) {
    let (token_contract_address, balances_storage_address) = match token {
        Token::STRK => (STRK_CONTRACT_ADDRESS.try_into().unwrap(), selector!("ERC20_balances")),
        Token::ETH => (ETH_CONTRACT_ADDRESS.try_into().unwrap(), selector!("ERC20_balances")),
        Token::Custom(CustomToken { address,
        balances_storage_address }) => (address, balances_storage_address),
    };
    let target_balance_storage_address = map_entry_address(
        balances_storage_address, array![target.into()].span()
    );
    let new_balance_felt = new_balance.try_into().unwrap();
    store(token_contract_address, target_balance_storage_address, array![new_balance_felt].span());
}

/// Returns the balance of `token` for `target` contract
/// - `target` - address of the contract, which balance you want to get
/// - `token` - token for which the balance is being retrieved
pub fn get_balance(target: ContractAddress, token: Token) -> u256 {
    let (token_contract_address, balances_storage_address) = match token {
        Token::STRK => (STRK_CONTRACT_ADDRESS.try_into().unwrap(), selector!("ERC20_balances")),
        Token::ETH => (ETH_CONTRACT_ADDRESS.try_into().unwrap(), selector!("ERC20_balances")),
        Token::Custom(CustomToken { address,
        balances_storage_address }) => (address, balances_storage_address),
    };
    let target_balance_storage_address = map_entry_address(
        balances_storage_address, array![target.into()].span()
    );
    let balance_felt = load(token_contract_address, target_balance_storage_address, 1.into());
    balance_felt.at(0).clone().into()
}
