use crate::helpers::block_explorer::LinkProvider;
use conversions::string::IntoPaddedHexStr;
use conversions::{padded_felt::PaddedFelt, serde::serialize::CairoSerialize};
use foundry_ui::Message;
use foundry_ui::styling;
use indoc::formatdoc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json::json;

use super::{command::CommandResponse, explorer_link::OutputLink};
use crate::response::cast_message::SncastMessage;

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
pub struct DeployResponse {
    pub contract_address: PaddedFelt,
    pub transaction_hash: PaddedFelt,
}

impl CommandResponse for DeployResponse {}

impl Message for SncastMessage<DeployResponse> {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Deployment completed")
            .blank_line()
            .field(
                "Contract Address",
                &self.command_response.contract_address.into_padded_hex_str(),
            )
            .field(
                "Transaction Hash",
                &self.command_response.transaction_hash.into_padded_hex_str(),
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

impl OutputLink for DeployResponse {
    const TITLE: &'static str = "deployment";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        formatdoc!(
            "
            contract: {}
            transaction: {}
            ",
            provider.contract(self.contract_address),
            provider.transaction(self.transaction_hash)
        )
    }
}
