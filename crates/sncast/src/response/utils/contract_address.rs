use crate::response::cast_message::SncastCommandMessage;
use conversions::padded_felt::PaddedFelt;
use conversions::{serde::serialize::CairoSerialize, string::IntoPaddedHexStr};
use foundry_ui::styling;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
pub struct ContractAddressResponse {
    pub contract_address: PaddedFelt,
}

impl SncastCommandMessage for ContractAddressResponse {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .field(
                "Contract Address",
                &self.contract_address.into_padded_hex_str(),
            )
            .build()
    }
}
