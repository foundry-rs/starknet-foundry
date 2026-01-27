mod account;
mod path;
mod transport;

// Re-export transport types needed by lib.rs
// TODO: Remove SpeculosHttpTransport and replace SncastLedgerTransport with LedgerTransport
// when test transport support is removed
pub use transport::{SncastLedgerTransport, SpeculosHttpTransport};

pub use account::{
    create_ledger_signer, get_ledger_public_key, ledger_account, ledger_account_with_app,
};

// Re-export path parsing for use in other modules
pub use path::parse_derivation_path;

use crate::response::cast_message::SncastCommandMessage;
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use coins_ledger::transports::LedgerAsync;
use foundry_ui::styling;
use serde::Serialize;
use starknet_rust::signers::ledger::LedgerStarknetApp;
use starknet_types_core::felt::Felt;

use transport::create_ledger_app;

#[derive(Args, Debug)]
#[command(about = "Interact with Ledger hardware wallet")]
pub struct Ledger {
    #[command(subcommand)]
    pub subcommand: LedgerSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum LedgerSubcommand {
    /// Get public key from Ledger device
    GetPublicKey(GetPublicKey),
    /// Sign a hash using Ledger device
    SignHash(SignHash),
    /// Get Starknet app version from Ledger device
    AppVersion(AppVersion),
}

#[derive(Args, Debug)]
pub struct GetPublicKey {
    /// Ledger derivation path in EIP-2645 format
    ///
    /// Supports extended format with string names and omitted 2645 prefix:
    ///   - Extended: m//starknet'/sncast'/0'/0'/0'
    ///   - Standard: m/2645'/1195502025'/355113700'/0'/0'/0'
    ///
    /// Recommended default: m//starknet'/sncast'/0'/0'/0'
    #[arg(long)]
    pub path: String,

    /// Do not display the public key on Ledger's screen for confirmation
    #[arg(long)]
    pub no_display: bool,
}

#[derive(Args, Debug)]
pub struct SignHash {
    /// Ledger derivation path in EIP-2645 format
    ///
    /// Supports extended format with string names and omitted 2645 prefix:
    ///   - Extended: m//starknet'/sncast'/0'/0'/0'
    ///   - Standard: m/2645'/1195502025'/355113700'/0'/0'/0'
    ///
    /// Recommended default: m//starknet'/sncast'/0'/0'/0'
    #[arg(long)]
    pub path: String,

    /// The raw hash to be signed (hex string with or without 0x prefix)
    pub hash: String,
}

#[derive(Args, Debug)]
pub struct AppVersion;

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LedgerResponse {
    PublicKey(PublicKeyResponse),
    Signature(SignatureResponse),
    Version(VersionResponse),
}

#[derive(Debug, Serialize)]
pub struct PublicKeyResponse {
    pub public_key: String,
}

#[derive(Debug, Serialize)]
pub struct SignatureResponse {
    pub signature: String,
}

#[derive(Debug, Serialize)]
pub struct VersionResponse {
    pub version: String,
}

impl SncastCommandMessage for LedgerResponse {
    fn text(&self) -> String {
        match self {
            LedgerResponse::PublicKey(resp) => styling::OutputBuilder::new()
                .field("Public Key", &resp.public_key)
                .build(),
            LedgerResponse::Signature(resp) => styling::OutputBuilder::new()
                .field("Signature", &resp.signature)
                .build(),
            LedgerResponse::Version(resp) => styling::OutputBuilder::new()
                .field("App Version", &resp.version)
                .build(),
        }
    }
}

pub async fn get_public_key<T: LedgerAsync + 'static>(
    args: &GetPublicKey,
    ledger: LedgerStarknetApp<T>,
) -> Result<LedgerResponse> {
    let path = parse_derivation_path(&args.path)?;

    if !args.no_display {
        eprintln!("Please confirm the public key on your Ledger");
    }

    let public_key = ledger
        .get_public_key(path, !args.no_display)
        .await
        .context("Failed to get public key from Ledger")?;

    Ok(LedgerResponse::PublicKey(PublicKeyResponse {
        public_key: format!("{:#066x}", public_key.scalar()),
    }))
}

pub async fn sign_hash<T: LedgerAsync + 'static>(
    args: &SignHash,
    ledger: LedgerStarknetApp<T>,
) -> Result<LedgerResponse> {
    let hash = Felt::from_hex(&args.hash)
        .with_context(|| format!("Failed to parse hash: {}", args.hash))?;

    let path = parse_derivation_path(&args.path)?;

    eprintln!(
        "WARNING: blind signing a raw hash could be dangerous. Make sure you ONLY sign hashes \
        from trusted sources. If you're sending transactions, use Ledger as a signer instead \
        of using this command."
    );

    eprintln!("Please confirm the signing operation on your Ledger");

    let signature = ledger
        .sign_hash(path, &hash)
        .await
        .context("Failed to sign hash with Ledger")?;

    Ok(LedgerResponse::Signature(SignatureResponse {
        signature: format!("0x{signature}"),
    }))
}

pub async fn app_version<T: LedgerAsync + 'static>(
    _args: &AppVersion,
    ledger: LedgerStarknetApp<T>,
) -> Result<LedgerResponse> {
    let version = ledger
        .get_version()
        .await
        .context("Failed to get app version from Ledger")?;

    Ok(LedgerResponse::Version(VersionResponse {
        version: version.to_string(),
    }))
}

pub async fn ledger(ledger_args: &Ledger) -> Result<LedgerResponse> {
    let ledger = create_ledger_app().await?;

    match &ledger_args.subcommand {
        LedgerSubcommand::GetPublicKey(args) => get_public_key(args, ledger).await,
        LedgerSubcommand::SignHash(args) => sign_hash(args, ledger).await,
        LedgerSubcommand::AppVersion(args) => app_version(args, ledger).await,
    }
}
