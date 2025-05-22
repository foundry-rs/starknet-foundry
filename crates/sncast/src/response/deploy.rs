use conversions::{padded_felt::PaddedFelt, serde::serialize::CairoSerialize};
use foundry_ui::{Message, formats::OutputFormat};
use indoc::formatdoc;
use serde::{Deserialize, Serialize};

use crate::helpers::block_explorer::LinkProvider;

use super::{
    cast_message::CastMessage,
    command::CommandResponse,
    explorer_link::OutputLink,
    print::{Format, OutputData},
};

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
pub struct DeployResponse {
    pub contract_address: PaddedFelt,
    pub transaction_hash: PaddedFelt,
}

impl CommandResponse for DeployResponse {}

impl Message for DeployResponse {}

impl CastMessage<DeployResponse> {
    // TODO(#3391): Update text output to be more user friendly
    #[must_use]
    pub fn text(&self) -> String {
        OutputData::from(&self.message)
            .format_with(self.numbers_format)
            .to_string_pretty("deploy", OutputFormat::Human)
            .expect("Failed to format response")
    }

    #[must_use]
    pub fn json(&self) -> String {
        OutputData::from(&self.message)
            .format_with(self.numbers_format)
            .to_string_pretty("deploy", OutputFormat::Json)
            .expect("Failed to format response")
    }
}

impl OutputLink for DeployResponse {
    const TITLE: &'static str = "deployment";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        formatdoc!(
            "
            contract: {}
            transaction: {}
            ",
            provider.contract(self.contract_address),
            provider.transaction(self.transaction_hash)
        )
    }
}
