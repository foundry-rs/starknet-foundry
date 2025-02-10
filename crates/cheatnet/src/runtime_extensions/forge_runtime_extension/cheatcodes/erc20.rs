use blockifier::state::state_api::State;
use conversions::serde::deserialize::CairoDeserialize;
use conversions::string::TryFromHexStr;
use conversions::IntoConv;
use starknet::core::crypto::pedersen_hash;
use starknet::core::types::u256::U256;
use starknet::core::utils::get_selector_from_name;
use starknet_api::core::ContractAddress;
use starknet_types_core::felt::Felt;

use super::storage::{load, normalize_storage_address, store};

const STRK_CONTRACT_ADDRESS: &str =
    "0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d";
const ETH_CONTRACT_ADDRESS: &str =
    "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7";

#[derive(CairoDeserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub enum Token {
    STRK,
    ETH,
    Custom {
        address: ContractAddress,
        balances_storage_address: Felt,
    },
}

///
/// # Arguments
///
/// * `blockifier_state`: Blockifier state reader
/// * `target`: The address of the contract we want to target
/// * `new_balance`: The new balance to set
/// * `token`: The token to set the balance for
///
/// returns: Result<(), Error> - a result containing the error if `set_balance` failed
pub fn set_balance(
    state: &mut dyn State,
    target: ContractAddress,
    new_balance: U256,
    token: Token,
) -> Result<(), anyhow::Error> {
    let (token_contract_address, balances_storage_address) = match token {
        Token::STRK => (
            ContractAddress::try_from_hex_str(STRK_CONTRACT_ADDRESS).unwrap(),
            get_selector_from_name("ERC20_balances").unwrap(),
        ),
        Token::ETH => (
            ContractAddress::try_from_hex_str(ETH_CONTRACT_ADDRESS).unwrap(),
            get_selector_from_name("ERC20_balances").unwrap(),
        ),
        Token::Custom {
            address,
            balances_storage_address,
        } => (address, balances_storage_address),
    };

    let target_balance_storage_address =
        normalize_storage_address(pedersen_hash(&balances_storage_address, &target.into()));
    store(
        state,
        token_contract_address,
        target_balance_storage_address,
        new_balance.into_(),
    )
}

pub fn get_balance(
    state: &mut dyn State,
    target: ContractAddress,
    token: Token,
) -> Result<U256, anyhow::Error> {
    let (token_contract_address, balances_storage_address) = match token {
        Token::STRK => (
            ContractAddress::try_from_hex_str(STRK_CONTRACT_ADDRESS).unwrap(),
            get_selector_from_name("ERC20_balances").unwrap(),
        ),
        Token::ETH => (
            ContractAddress::try_from_hex_str(ETH_CONTRACT_ADDRESS).unwrap(),
            get_selector_from_name("ERC20_balances").unwrap(),
        ),
        Token::Custom {
            address,
            balances_storage_address,
        } => (address, balances_storage_address),
    };

    let target_balance_storage_address =
        normalize_storage_address(pedersen_hash(&balances_storage_address, &target.into()));
    let balance = load(
        state,
        token_contract_address,
        target_balance_storage_address,
    )?;
    Ok(U256::from(balance))
}
