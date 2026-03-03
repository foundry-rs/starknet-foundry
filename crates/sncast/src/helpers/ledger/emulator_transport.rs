/// This module is only used behind the `ledger-emulator` feature flag.
/// It is used to test the ledger commands locally without having to connect to a real Ledger device.
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

#[derive(Debug)]
pub struct SpeculosHttpTransport {
    client: Client,
    base_url: String,
}

#[derive(Serialize)]
struct ApduRequest {
    data: String,
}

fn io_err(e: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> LedgerError {
    LedgerError::NativeTransportError(NativeTransportError::Io(IoError::other(e)))
}

impl SpeculosHttpTransport {
    pub fn new(url: String) -> Result<Self, LedgerError> {
        let client = Client::builder().build().map_err(io_err)?;
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
        let request = ApduRequest {
            data: const_hex::encode(command.serialize()),
        };

        let resp = self
            .client
            .post(format!("{}/apdu", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(io_err)?;

        if !resp.status().is_success() {
            return Err(io_err(format!("HTTP Error: {}", resp.status())));
        }

        let body: serde_json::Value = resp.json().await.map_err(io_err)?;

        if let Some(err) = body.get("error").and_then(|v| v.as_str()) {
            return Err(io_err(format!("Speculos Error: {err}")));
        }

        let data_str =
            body.get("data")
                .and_then(|v| v.as_str())
                .ok_or(LedgerError::NativeTransportError(
                    NativeTransportError::Comm("Missing data field"),
                ))?;

        let answer = const_hex::decode(data_str).map_err(io_err)?;

        APDUAnswer::from_answer(answer).map_err(|_| {
            LedgerError::NativeTransportError(NativeTransportError::Comm("Invalid APDU response"))
        })
    }

    fn close(self) {}
}

pub fn emulator_ledger_app() -> Result<LedgerStarknetApp<SpeculosHttpTransport>, StarknetLedgerError>
{
    let url = std::env::var("LEDGER_EMULATOR_URL")
        .context("LEDGER_EMULATOR_URL must be set to a Speculos URL when using the ledger-emulator feature")
        .map_err(io_err)?;
    let transport = SpeculosHttpTransport::new(url.clone()).map_err(io_err)?;
    Ok(LedgerStarknetApp::from_transport(transport))
}
