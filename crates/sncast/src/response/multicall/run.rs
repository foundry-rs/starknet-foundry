use crate::response::cast_message::SncastCommandMessage;
use crate::{
    helpers::block_explorer::LinkProvider,
    response::{dry_run::DryRunResponse, explorer_link::OutputLink, invoke::InvokeResponse},
};
use conversions::string::IntoHexStr;
use conversions::{padded_felt::PaddedFelt, serde::serialize::CairoSerialize};
use foundry_ui::styling;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, CairoSerialize, Clone, Debug, PartialEq)]
pub enum MulticallRunResponse {
    Transaction(MulticallRunTransactionResponse),
    DryRun(DryRunResponse),
}

#[derive(Serialize, Deserialize, CairoSerialize, Clone, Debug, PartialEq)]
pub struct MulticallRunTransactionResponse {
    pub transaction_hash: PaddedFelt,
}

impl SncastCommandMessage for MulticallRunResponse {
    fn text(&self) -> String {
        match self {
            MulticallRunResponse::Transaction(response) => response.text(),
            MulticallRunResponse::DryRun(response) => response.text(),
        }
    }

    fn json(&self) -> serde_json::Value {
        match self {
            MulticallRunResponse::Transaction(response) => response.json(),
            MulticallRunResponse::DryRun(response) => response.json(),
        }
    }
}

impl SncastCommandMessage for MulticallRunTransactionResponse {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Multicall completed")
            .blank_line()
            .field("Transaction Hash", &self.transaction_hash.into_hex_string())
            .build()
    }
}

impl From<InvokeResponse> for MulticallRunResponse {
    fn from(value: InvokeResponse) -> Self {
        match value {
            InvokeResponse::Transaction(invoke_response) => {
                MulticallRunResponse::Transaction(MulticallRunTransactionResponse {
                    transaction_hash: invoke_response.transaction_hash,
                })
            }
            InvokeResponse::DryRun(response) => MulticallRunResponse::DryRun(response),
        }
    }
}

impl OutputLink for MulticallRunResponse {
    const TITLE: &'static str = "invocation";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        match self {
            MulticallRunResponse::Transaction(response) => format!(
                "transaction: {}",
                provider.transaction(response.transaction_hash)
            ),
            MulticallRunResponse::DryRun(_) => {
                unreachable!("Dry run response should not generate explorer links")
            }
        }
    }

    fn is_dry_run(&self) -> bool {
        matches!(self, MulticallRunResponse::DryRun(_))
    }
}
