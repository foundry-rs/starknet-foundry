use crate::{
    helpers::block_explorer::LinkProvider,
    response::{
        cast_message::SncastMessage, command::CommandResponse, explorer_link::OutputLink,
        invoke::InvokeResponse,
    },
};
use conversions::string::IntoHexStr;
use conversions::{padded_felt::PaddedFelt, serde::serialize::CairoSerialize};
use foundry_ui::Message;
use foundry_ui::styling;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json::json;

#[derive(Serialize, Deserialize, CairoSerialize, Clone, Debug, PartialEq)]
pub struct MulticallRunResponse {
    pub transaction_hash: PaddedFelt,
}

impl CommandResponse for MulticallRunResponse {}

impl Message for SncastMessage<MulticallRunResponse> {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Multicall completed")
            .blank_line()
            .field(
                "Transaction Hash",
                &self.command_response.transaction_hash.into_hex_string(),
            )
            .build()
    }

    fn json(&self) -> Value {
        serde_json::to_value(&self.command_response).unwrap_or_else(|err| {
            json!({
                "error": "Failed to serialize response",
                "command": self.command,
                "details": err.to_string()
            })
        })
    }
}

impl From<InvokeResponse> for MulticallRunResponse {
    fn from(value: InvokeResponse) -> Self {
        Self {
            transaction_hash: value.transaction_hash,
        }
    }
}

impl OutputLink for MulticallRunResponse {
    const TITLE: &'static str = "invocation";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        format!(
            "transaction: {}",
            provider.transaction(self.transaction_hash)
        )
    }
}
