use crate::response::cast_message::SncastCommandMessage;
use foundry_ui::styling;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PublicKeyResponse {
    pub public_key: String,
}

#[derive(Debug, Serialize)]
pub struct SignatureResponse {
    pub r: String,
    pub s: String,
}

#[derive(Debug, Serialize)]
pub struct VersionResponse {
    pub version: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LedgerResponse {
    PublicKey(PublicKeyResponse),
    Signature(SignatureResponse),
    Version(VersionResponse),
}

impl SncastCommandMessage for LedgerResponse {
    fn text(&self) -> String {
        match self {
            LedgerResponse::PublicKey(resp) => styling::OutputBuilder::new()
                .field("Public Key", &resp.public_key)
                .build(),
            LedgerResponse::Signature(resp) => styling::OutputBuilder::new()
                .text_field("Hash signature:")
                .field("r", &resp.r)
                .field("s", &resp.s)
                .build(),
            LedgerResponse::Version(resp) => styling::OutputBuilder::new()
                .field("App Version", &resp.version)
                .build(),
        }
    }
}
