use super::{command::CommandResponse, explorer_link::OutputLink};
use crate::helpers::block_explorer::LinkProvider;
use crate::response::cast_message::{SncastCommandMessage, SncastMessage};
use conversions::string::IntoPaddedHexStr;
use conversions::{padded_felt::PaddedFelt, serde::serialize::CairoSerialize};
use foundry_ui::styling;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, CairoSerialize, Clone, Debug, PartialEq)]
pub struct InvokeResponse {
    pub transaction_hash: PaddedFelt,
}

impl CommandResponse for InvokeResponse {}

impl SncastCommandMessage for SncastMessage<InvokeResponse> {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Invoke completed")
            .blank_line()
            .field(
                "Transaction Hash",
                &self.command_response.transaction_hash.into_padded_hex_str(),
            )
            .build()
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
