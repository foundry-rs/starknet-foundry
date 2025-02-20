use blockifier::state::state_api::State;
use conversions::serde::deserialize::CairoDeserialize;
use conversions::string::TryFromHexStr;
use starknet::core::{types::U256, utils::get_selector_from_name};
use starknet_api::core::ContractAddress;
use starknet_types_core::felt::Felt;

use super::storage::{calculate_variable_address, store};

const STRK_CONTRACT_ADDRESS: &str =
    "0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d";

#[derive(CairoDeserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub enum Token {
    STRK,
    Custom {
        contract_address: ContractAddress,
        balances_variable_selector: Felt,
    },
}

impl Token {
    #[must_use]
    pub fn contract_address(&self) -> ContractAddress {
        match self {
            Token::STRK => TryFromHexStr::try_from_hex_str(STRK_CONTRACT_ADDRESS).unwrap(),
            Token::Custom {
                contract_address, ..
            } => *contract_address,
        }
    }

    #[must_use]
    pub fn balances_variable_selector(&self) -> Felt {
        match self {
            Token::STRK => get_selector_from_name("ERC20_balances").unwrap(),
            Token::Custom {
                balances_variable_selector,
                ..
            } => *balances_variable_selector,
        }
    }
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
    let balance_low_address =
        calculate_variable_address(token.balances_variable_selector(), Some(&[target.into()]));
    let balance_high_address = balance_low_address + Felt::ONE;
    store(
        state,
        token.contract_address(),
        balance_low_address,
        new_balance.low().into(),
    )?;
    store(
        state,
        token.contract_address(),
        balance_high_address,
        new_balance.high().into(),
    )?;
    Ok(())
}
