use crate::helpers::block_explorer::LinkProvider;

use super::{command::CommandResponse, explorer_link::OutputLink};
use conversions::string::IntoPaddedHexStr;
use conversions::{padded_felt::PaddedFelt, serde::serialize::CairoSerialize};
use foundry_ui::Message;
use foundry_ui::styling;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::response::cast_message::SncastMessage;

#[derive(Serialize, Deserialize, CairoSerialize, Clone, Debug, PartialEq)]
pub struct InvokeResponse {
    pub transaction_hash: PaddedFelt,
}

impl CommandResponse for InvokeResponse {}

impl Message for SncastMessage<InvokeResponse> {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Invocation completed")
            .blank_line()
            .field(
                "Transaction Hash",
                &self.command_response.transaction_hash.into_padded_hex_str(),
            )
            .build()
    }

    fn json(&self) -> Value {
        serde_json::to_value(&self.command_response).unwrap()
    }
}

impl OutputLink for InvokeResponse {
    const TITLE: &'static str = "invocation";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        format!(
            "transaction: {}",
            provider.transaction(self.transaction_hash)
        )
    }
}
