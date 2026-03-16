use anyhow::anyhow;
use clap::Args;
use conversions::IntoConv;
use sncast::response::{errors::StarknetCommandError, utils::selector::SelectorResponse};
use starknet_rust::core::utils::get_selector_from_name;

#[derive(Args, Debug)]
#[command(about = "Calculate selector from name", long_about = None)]
pub struct Selector {
    /// Selector name
    pub name: String,
}

#[allow(clippy::result_large_err)]
pub fn get_selector(selector: &Selector) -> Result<SelectorResponse, StarknetCommandError> {
    let trimmed = selector.name.trim();

    if trimmed.contains('(') || trimmed.contains(')') {
        return Err(StarknetCommandError::UnknownError(anyhow!(
            "Parentheses and the content within should not be supplied"
        )));
    }

    let felt = get_selector_from_name(trimmed)
        .map_err(|e| StarknetCommandError::UnknownError(anyhow::Error::from(e)))?;

    Ok(SelectorResponse {
        selector: felt.into_(),
    })
}
