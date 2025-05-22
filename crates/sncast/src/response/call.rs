use super::command::CommandResponse;
use conversions::serde::serialize::CairoSerialize;
use serde::Serialize;
use starknet_types_core::felt::Felt;

#[derive(Serialize, CairoSerialize, Clone)]
pub struct CallResponse {
    pub response: Vec<Felt>,
}

impl CommandResponse for CallResponse {}

// TODO(#3391): Update text output to be more user friendly
// impl Message for CastMessage<CallResponse> { }
