use conversions::padded_felt::PaddedFelt;
use foundry_ui::styling::OutputBuilder;
use serde::Serialize;
use starknet_rust::core::types::ContractClass;
use starknet_rust_crypto::Felt;

use crate::response::{
    cast_message::SncastCommandMessage,
    explorer_link::{ExplorerLink, ExplorerLinkBuilder, OutputLink},
};

#[derive(Clone, Serialize)]
pub struct ClassResponse {
    #[serde(flatten)]
    pub response: ContractClass,
    #[serde(skip)]
    pub class_hash: Felt,
}

impl SncastCommandMessage for ClassResponse {
    fn text(&self) -> String {
        match serde_json::to_string_pretty(self) {
            Ok(res) => res,
            Err(e) => OutputBuilder::new()
                .error_message(&format!("Failed to serialize response: {e}"))
                .build(),
        }
    }
}

impl OutputLink for ClassResponse {
    const TITLE: &str = "class";

    fn get_links(
        &self,
        provider: Box<dyn crate::helpers::block_explorer::LinkProvider>,
    ) -> Vec<ExplorerLink> {
        ExplorerLinkBuilder::new()
            .link("class", provider.class(PaddedFelt(self.class_hash)))
            .build()
    }
}
