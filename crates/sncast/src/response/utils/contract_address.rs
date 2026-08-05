use crate::response::cast_message::SncastCommandMessage;
use conversions::padded_felt::PaddedFelt;
use conversions::string::IntoPaddedHexStr;
use foundry_ui::styling;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct ContractAddressResponse {
    pub contract_address: PaddedFelt,
    pub salt: PaddedFelt,
}

impl SncastCommandMessage for ContractAddressResponse {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .field(
                "Contract Address",
                &self.contract_address.into_padded_hex_str(),
            )
            .field("Salt", &self.salt.into_padded_hex_str())
            .build()
    }
}
