use crate::response::helpers::serialize_json;
use crate::{
    helpers::token::Token,
    response::{cast_message::SncastMessage, command::CommandResponse},
};
use foundry_ui::{Message, styling};
use serde::Serialize;

use starknet::core::types::U256;

#[derive(Serialize, Debug)]
pub struct BalanceResponse {
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
            .field("Balance", &balance_str)
            .build()
    }

    fn json(&self) -> serde_json::Value {
        serialize_json(self)
    }
}
