use super::command::CommandResponse;
use serde::Serialize;
use starknet_types_core::felt::Felt;

#[derive(Serialize, Clone)]
pub struct TransformedCallResponse {
    pub response: String,
    pub response_raw: Vec<Felt>,
}

impl CommandResponse for TransformedCallResponse {}

// TODO(#3391): Update text output to be more user friendly
// impl Message for CastMessage<TransformedCallResponse> {}
