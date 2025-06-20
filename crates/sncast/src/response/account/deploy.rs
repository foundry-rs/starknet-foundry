use crate::response::cast_message::SncastMessage;
use crate::{
    helpers::block_explorer::LinkProvider,
    response::{command::CommandResponse, explorer_link::OutputLink, invoke::InvokeResponse},
};
use conversions::string::IntoHexStr;
use conversions::{padded_felt::PaddedFelt, serde::serialize::CairoSerialize};
use foundry_ui::Message;
use foundry_ui::styling;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, CairoSerialize, Clone, Debug, PartialEq)]
pub struct AccountDeployResponse {
    pub transaction_hash: PaddedFelt,
}

impl CommandResponse for AccountDeployResponse {}
impl From<InvokeResponse> for AccountDeployResponse {
    fn from(value: InvokeResponse) -> Self {
        Self {
            transaction_hash: value.transaction_hash,
        }
    }
}

impl Message for SncastMessage<AccountDeployResponse> {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Account successfully deployed")
            .blank_line()
            .field(
                "Transaction Hash",
                &self.command_response.transaction_hash.into_hex_string(),
            )
            .build()
    }

    fn json(&self) -> Value {
        serde_json::to_value(&self.command_response).unwrap()
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
