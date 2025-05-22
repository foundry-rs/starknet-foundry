use conversions::{padded_felt::PaddedFelt, serde::serialize::CairoSerialize};
use foundry_ui::{Message, OutputFormat};
use serde::{Deserialize, Serialize};

use crate::{
    helpers::block_explorer::LinkProvider,
    response::{
        cast_message::CastMessage,
        command::CommandResponse,
        explorer_link::OutputLink,
        invoke::InvokeResponse,
        print::{Format, OutputData},
    },
};

#[derive(Serialize, Deserialize, CairoSerialize, Clone, Debug, PartialEq)]
pub struct AccountDeployResponse {
    pub transaction_hash: PaddedFelt,
}

impl CommandResponse for AccountDeployResponse {}

impl Message for AccountDeployResponse {}

impl CastMessage<AccountDeployResponse> {
    // TODO(#3391): Update text output to be more user friendly
    #[must_use]
    pub fn text(&self) -> String {
        OutputData::from(&self.message)
            .format_with(self.numbers_format)
            .to_string_pretty("account deploy", OutputFormat::Human)
            .expect("Failed to format response")
    }

    #[must_use]
    pub fn json(&self) -> String {
        OutputData::from(&self.message)
            .format_with(self.numbers_format)
            .to_string_pretty("account deploy", OutputFormat::Json)
            .expect("Failed to format response")
    }
}

impl From<InvokeResponse> for AccountDeployResponse {
    fn from(value: InvokeResponse) -> Self {
        Self {
            transaction_hash: value.transaction_hash,
        }
    }
}

impl OutputLink for AccountDeployResponse {
    const TITLE: &'static str = "invocation";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        format!(
            "transaction: {}",
            provider.transaction(self.transaction_hash)
        )
    }
}
