use crate::response::cast_message::{SncastCommandMessage, SncastMessage};
use crate::{
    helpers::block_explorer::LinkProvider,
    response::{explorer_link::OutputLink, invoke::InvokeResponse},
};
use conversions::string::IntoHexStr;
use conversions::{padded_felt::PaddedFelt, serde::serialize::CairoSerialize};
use foundry_ui::styling;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, CairoSerialize, Clone, Debug, PartialEq)]
pub struct AccountDeployResponse {
    pub transaction_hash: PaddedFelt,
}

impl From<InvokeResponse> for AccountDeployResponse {
    fn from(value: InvokeResponse) -> Self {
        Self {
            transaction_hash: value.transaction_hash,
        }
    }
}

impl SncastCommandMessage for SncastMessage<AccountDeployResponse> {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Account deployed")
            .blank_line()
            .field(
                "Transaction Hash",
                &self.command_response.transaction_hash.into_hex_string(),
            )
            .build()
    }
}

impl OutputLink for AccountDeployResponse {
    const TITLE: &'static str = "account deployment";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        format!(
            "transaction: {}",
            provider.transaction(self.transaction_hash)
        )
    }
}
