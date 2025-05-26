use conversions::{padded_felt::PaddedFelt, serde::serialize::CairoSerialize};
use indoc::formatdoc;
use serde::{Deserialize, Serialize};

use crate::helpers::block_explorer::LinkProvider;

use super::{command::CommandResponse, explorer_link::OutputLink};

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
pub struct DeployResponse {
    pub contract_address: PaddedFelt,
    pub transaction_hash: PaddedFelt,
}

impl CommandResponse for DeployResponse {}

// TODO(#3391): Update text output to be more user friendly
// impl Message for SncastMessage<DeployResponse> {}

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
