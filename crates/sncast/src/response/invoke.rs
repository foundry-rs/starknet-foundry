use crate::helpers::block_explorer::LinkProvider;

use super::{command::CommandResponse, explorer_link::OutputLink};
use conversions::{padded_felt::PaddedFelt, serde::serialize::CairoSerialize};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, CairoSerialize, Clone, Debug, PartialEq)]
pub struct InvokeResponse {
    pub transaction_hash: PaddedFelt,
}

impl CommandResponse for InvokeResponse {}

// TODO(#3391): Update text output to be more user friendly
// impl Message for SnastMessage<InvokeResponse> { }

impl OutputLink for InvokeResponse {
    const TITLE: &'static str = "invocation";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        format!(
            "transaction: {}",
            provider.transaction(self.transaction_hash)
        )
    }
}
