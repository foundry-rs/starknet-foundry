use foundry_ui::styling::OutputBuilder;
use serde::Serialize;
use starknet_rust_crypto::Felt;

use crate::response::cast_message::SncastCommandMessage;

#[derive(Serialize)]
pub struct ChainIdResponse {
    pub chain_id: Felt,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_name: Option<String>,
}

impl SncastCommandMessage for ChainIdResponse {
    fn text(&self) -> String {
        OutputBuilder::new()
            .success_message("Chain ID retrieved")
            .blank_line()
            .felt_field("Chain ID", &self.chain_id)
            .if_some(self.chain_name.as_ref(), |builder, name| {
                builder.field("Chain Name", name)
            })
            .build()
    }
}
