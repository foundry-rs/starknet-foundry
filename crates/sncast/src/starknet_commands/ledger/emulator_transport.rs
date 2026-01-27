use anyhow::Context;
use async_trait::async_trait;
use coins_ledger::{
    APDUAnswer, APDUCommand, LedgerError,
    transports::{LedgerAsync, native::NativeTransportError},
};
use reqwest::Client;
use serde::Serialize;
use starknet_rust::signers::ledger::{LedgerError as StarknetLedgerError, LedgerStarknetApp};
use std::io::Error as IoError;

/// HTTP transport for the Speculos Ledger emulator.
#[derive(Debug)]
pub struct SpeculosHttpTransport {
    client: Client,
    base_url: String,
}

#[derive(Serialize)]
struct ApduRequest {
    data: String,
}

impl SpeculosHttpTransport {
    pub fn new(url: String) -> Result<Self, LedgerError> {
        let client = Client::builder().build().map_err(|e| {
            LedgerError::NativeTransportError(NativeTransportError::Io(IoError::other(e)))
        })?;
        Ok(Self {
            client,
            base_url: url,
        })
    }
}

#[async_trait]
impl LedgerAsync for SpeculosHttpTransport {
    async fn init() -> Result<Self, LedgerError> {
        Self::new("http://127.0.0.1:5001".to_string())
    }

    async fn exchange(&self, command: &APDUCommand) -> Result<APDUAnswer, LedgerError> {
        let hex_command = const_hex::encode(command.serialize());
        let request = ApduRequest { data: hex_command };

        let resp = self
            .client
            .post(format!("{}/apdu", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                LedgerError::NativeTransportError(NativeTransportError::Io(IoError::other(e)))
            })?;

        if !resp.status().is_success() {
            return Err(LedgerError::NativeTransportError(NativeTransportError::Io(
                IoError::other(format!("HTTP Error: {}", resp.status())),
            )));
        }

        let apdu_resp: serde_json::Value = resp.json().await.map_err(|e| {
            LedgerError::NativeTransportError(NativeTransportError::Io(IoError::other(e)))
        })?;

        if let Some(err) = apdu_resp.get("error").and_then(|v| v.as_str()) {
            return Err(LedgerError::NativeTransportError(NativeTransportError::Io(
                IoError::other(format!("Speculos Error: {err}")),
            )));
        }

        let data_str = apdu_resp
            .get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                LedgerError::NativeTransportError(NativeTransportError::Comm("Missing data field"))
            })?;

        let data = const_hex::decode(data_str).map_err(|e| {
            LedgerError::NativeTransportError(NativeTransportError::Io(IoError::other(e)))
        })?;

        APDUAnswer::from_answer(data).map_err(|_| {
            LedgerError::NativeTransportError(NativeTransportError::Comm("Invalid APDU response"))
        })
    }

    fn close(self) {}
}

pub fn emulator_ledger_app() -> Result<LedgerStarknetApp<SpeculosHttpTransport>, StarknetLedgerError>
{
    let url = std::env::var("LEDGER_EMULATOR_URL").context(
        "LEDGER_EMULATOR_URL must be set to a Speculos URL when using the ledger-emulator feature",
    ).map_err(|e| LedgerError::NativeTransportError(NativeTransportError::Io(IoError::other(e))))?;
    let transport = SpeculosHttpTransport::new(url.clone()).map_err(|e| {
        LedgerError::NativeTransportError(NativeTransportError::Io(IoError::other(e)))
    })?;
    Ok(LedgerStarknetApp::from_transport(transport))
}
