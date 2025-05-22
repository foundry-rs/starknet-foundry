use conversions::{padded_felt::PaddedFelt, serde::serialize::CairoSerialize};
use serde::{Deserialize, Serialize};

use crate::{
    helpers::block_explorer::LinkProvider,
    response::{command::CommandResponse, explorer_link::OutputLink, invoke::InvokeResponse},
};

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

// TODO(#3391): Update text output to be more user friendly
// impl Message for SnastMessage<AccountCreateResponse> {}

impl OutputLink for AccountDeployResponse {
    const TITLE: &'static str = "invocation";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        format!(
            "transaction: {}",
            provider.transaction(self.transaction_hash)
        )
    }
}
