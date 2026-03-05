use super::explorer_link::OutputLink;
use crate::helpers::block_explorer::LinkProvider;
use crate::response::cast_message::SncastCommandMessage;
use crate::response::dry_run::DryRunResponse;
use conversions::string::IntoPaddedHexStr;
use conversions::{padded_felt::PaddedFelt, serde::serialize::CairoSerialize};
use foundry_ui::styling;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, CairoSerialize, Clone, Debug, PartialEq)]
pub enum InvokeResponse {
    Transaction(InvokeTransactionResponse),
    DryRun(DryRunResponse),
}

#[derive(Serialize, Deserialize, CairoSerialize, Clone, Debug, PartialEq)]
pub struct InvokeTransactionResponse {
    pub transaction_hash: PaddedFelt,
}

impl SncastCommandMessage for InvokeResponse {
    fn text(&self) -> String {
        match self {
            InvokeResponse::Transaction(response) => styling::OutputBuilder::new()
                .success_message("Invoke completed")
                .blank_line()
                .field(
                    "Transaction Hash",
                    &response.transaction_hash.into_padded_hex_str(),
                )
                .build(),
            InvokeResponse::DryRun(response) => response.text(),
        }
    }
}

impl OutputLink for InvokeTransactionResponse {
    const TITLE: &'static str = "invocation";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        format!(
            "transaction: {}",
            provider.transaction(self.transaction_hash)
        )
    }
}
