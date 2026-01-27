// TODO: This entire file can be removed when test transport support (SNCAST_TEST_LEDGER) is removed.
// After removal, replace all SncastLedgerTransport usages with LedgerTransport directly
// and simplify create_ledger_app to just use LedgerTransport::init().

use anyhow::{Context, Result};
use async_trait::async_trait;
use coins_ledger::{
    APDUAnswer, APDUCommand, LedgerError,
    transports::{Ledger as LedgerTransport, LedgerAsync, native::NativeTransportError},
};
use reqwest::Client;
use serde::Serialize;
use starknet_rust::signers::ledger::LedgerStarknetApp;
use std::io::Error as IoError;

/// HTTP transport for Speculos emulator (used in tests)
///
/// This type is public because it appears in type signatures, but users typically
/// don't need to construct it directly. Use `ledger_account` or `create_ledger_signer` instead.
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

/// Transport enum that wraps both HID (real Ledger device) and HTTP (Speculos emulator) transports
///
/// This type is public because it appears in type signatures, but users typically
/// don't need to construct it directly. Use `ledger_account` or `create_ledger_signer` instead.
#[derive(Debug)]
pub enum SncastLedgerTransport {
    /// Real Ledger hardware device via HID
    Hid(LedgerTransport),
    /// Speculos emulator via HTTP (for testing)
    Http(SpeculosHttpTransport),
}

#[async_trait]
impl LedgerAsync for SncastLedgerTransport {
    async fn init() -> Result<Self, LedgerError> {
        LedgerTransport::init()
            .await
            .map(SncastLedgerTransport::Hid)
    }

    async fn exchange(&self, command: &APDUCommand) -> Result<APDUAnswer, LedgerError> {
        match self {
            SncastLedgerTransport::Hid(t) => t.exchange(command).await,
            SncastLedgerTransport::Http(t) => t.exchange(command).await,
        }
    }

    fn close(self) {
        match self {
            SncastLedgerTransport::Hid(t) => t.close(),
            SncastLedgerTransport::Http(t) => t.close(),
        }
    }
}

/// Internal helper to create a Ledger app connection
/// Handles both real Ledger devices and Speculos emulator
pub(super) async fn create_ledger_app() -> Result<LedgerStarknetApp<SncastLedgerTransport>> {
    if std::env::var("SNCAST_TEST_LEDGER").is_ok_and(|v| v == "1") {
        let transport = SpeculosHttpTransport::new("http://127.0.0.1:5001".to_string())
            .context("Failed to connect to Ledger emulator at http://127.0.0.1:5001")?;
        Ok(LedgerStarknetApp::from_transport(
            SncastLedgerTransport::Http(transport),
        ))
    } else {
        let transport = LedgerTransport::init()
            .await
            .context("Failed to connect to Ledger device. Make sure your Ledger is connected and the Starknet app is open.")?;
        Ok(LedgerStarknetApp::from_transport(
            SncastLedgerTransport::Hid(transport),
        ))
    }
}
