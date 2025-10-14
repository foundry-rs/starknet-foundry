use crate::{
    helpers::token::Token,
    response::{cast_message::SncastMessage, command::CommandResponse},
};
use conversions::{serde::serialize::CairoSerialize, string::IntoPaddedHexStr};
use foundry_ui::{Message, styling};
use serde::{Deserialize, Serialize};
use serde_json::json;
use starknet::core::types::U256;
use starknet_types_core::felt::Felt;

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
pub struct BalanceResponse {
    pub account_address: Felt,
    pub balance: (u128, u128),
    pub token: Option<Token>,
}

impl CommandResponse for BalanceResponse {}

impl Message for SncastMessage<BalanceResponse> {
    fn text(&self) -> String {
        let (low, high) = self.command_response.balance;
        let balance = U256::from_words(low, high).to_string();
        let balance_str = if let Some(token) = self.command_response.token {
            format!("{balance} {token}")
        } else {
            balance
        };

        styling::OutputBuilder::new()
            .field(
                "Account Address",
                &self.command_response.account_address.into_padded_hex_str(),
            )
            .field("Balance", &balance_str)
            .build()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::to_value(&self.command_response).unwrap_or_else(|err| {
            json!({
                "error": "Failed to serialize response",
                "command": self.command,
                "details": err.to_string()
            })
        })
    }
}
