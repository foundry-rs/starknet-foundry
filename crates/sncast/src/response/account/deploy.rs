use crate::response::cast_message::SncastCommandMessage;
use crate::response::invoke::InvokeTransactionResponse;
use crate::{
    helpers::block_explorer::LinkProvider,
    response::{dry_run::DryRunResponse, explorer_link::OutputLink, invoke::InvokeResponse},
};
use conversions::string::IntoHexStr;
use conversions::{padded_felt::PaddedFelt, serde::serialize::CairoSerialize};
use foundry_ui::styling;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, CairoSerialize, Clone, Debug, PartialEq)]
pub enum AccountDeployResponse {
    Transaction(InvokeTransactionResponse),
    DryRun(DryRunResponse),
}

#[derive(Serialize, Deserialize, CairoSerialize, Clone, Debug, PartialEq)]
pub struct AccountDeployTransactionResponse {
    pub transaction_hash: PaddedFelt,
}

impl From<InvokeResponse> for AccountDeployResponse {
    fn from(value: InvokeResponse) -> Self {
        match value {
            InvokeResponse::Transaction(invoke_response) => {
                AccountDeployResponse::Transaction(InvokeTransactionResponse {
                    transaction_hash: invoke_response.transaction_hash,
                })
            }
            InvokeResponse::DryRun(response) => AccountDeployResponse::DryRun(response),
        }
    }
}

impl SncastCommandMessage for AccountDeployResponse {
    fn text(&self) -> String {
        match self {
            AccountDeployResponse::Transaction(response) => styling::OutputBuilder::new()
                .success_message("Account deployed")
                .blank_line()
                .field(
                    "Transaction Hash",
                    &response.transaction_hash.into_hex_string(),
                )
                .build(),
            AccountDeployResponse::DryRun(response) => response.text(),
        }
    }

    fn json(&self) -> serde_json::Value {
        match self {
            AccountDeployResponse::Transaction(response) => response.json(),
            AccountDeployResponse::DryRun(response) => response.json(),
        }
    }
}

impl OutputLink for AccountDeployResponse {
    const TITLE: &'static str = "account deployment";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        match self {
            AccountDeployResponse::Transaction(response) => format!(
                "transaction: {}",
                provider.transaction(response.transaction_hash)
            ),
            AccountDeployResponse::DryRun(_) => {
                unreachable!("Dry run response should not generate explorer links")
            }
        }
    }
}
