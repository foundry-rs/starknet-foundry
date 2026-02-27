use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use coins_ledger::transports::LedgerAsync;
use conversions::string::IntoHexStr;
use foundry_ui::components::warning::WarningMessage;
use foundry_ui::styling;
use serde::Serialize;
use sncast::helpers::ledger::{DerivationPathParser, create_ledger_app};
use sncast::response::cast_message::SncastCommandMessage;
use sncast::response::ui::UI;
use starknet_rust::signers::DerivationPath;
use starknet_rust::signers::ledger::LedgerStarknetApp;
use starknet_types_core::felt::Felt;

#[derive(Args, Debug)]
#[command(about = "Interact with Ledger hardware wallet")]
pub struct Ledger {
    #[command(subcommand)]
    subcommand: LedgerSubcommand,
}

#[derive(Subcommand, Debug)]
enum LedgerSubcommand {
    /// Get public key from Ledger device
    GetPublicKey(GetPublicKey),
    /// Sign a hash using Ledger device
    SignHash(SignHash),
    /// Get Starknet app version from Ledger device
    AppVersion(AppVersion),
}

#[derive(Args, Debug)]
struct GetPublicKey {
    /// Ledger derivation path in EIP-2645 format
    #[arg(long, value_parser = DerivationPathParser)]
    path: DerivationPath,

    /// Do not display the public key on Ledger's screen for confirmation
    #[arg(long)]
    no_display: bool,
}

#[derive(Args, Debug)]
struct SignHash {
    /// Ledger derivation path in EIP-2645 format
    #[arg(long, value_parser = DerivationPathParser)]
    path: DerivationPath,

    /// The raw hash to be signed (hex string with or without 0x prefix)
    hash: String,
}

#[derive(Args, Debug)]
struct AppVersion;

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LedgerResponse {
    PublicKey(PublicKeyResponse),
    Signature(SignatureResponse),
    Version(VersionResponse),
}

#[derive(Debug, Serialize)]
pub struct PublicKeyResponse {
    public_key: String,
}

#[derive(Debug, Serialize)]
pub struct SignatureResponse {
    signature: String,
}

#[derive(Debug, Serialize)]
pub struct VersionResponse {
    version: String,
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

async fn get_public_key<T: LedgerAsync + 'static>(
    args: &GetPublicKey,
    ledger: LedgerStarknetApp<T>,
    ui: &UI,
) -> Result<LedgerResponse> {
    if !args.no_display {
        ui.print_notification("Please confirm the public key on your Ledger".to_string());
    }

    let public_key = ledger
        .get_public_key(args.path.clone(), !args.no_display)
        .await
        .context("Failed to get public key from Ledger")?;

    Ok(LedgerResponse::PublicKey(PublicKeyResponse {
        public_key: public_key.scalar().into_hex_string(),
    }))
}

async fn sign_hash<T: LedgerAsync + 'static>(
    args: &SignHash,
    ledger: LedgerStarknetApp<T>,
    ui: &UI,
) -> Result<LedgerResponse> {
    let hash = Felt::from_hex(&args.hash)
        .with_context(|| format!("Failed to parse hash: {}", args.hash))?;

    ui.print_warning(WarningMessage::new(
        "Blind signing a raw hash could be dangerous. Make sure you ONLY sign hashes \
        from trusted sources. If you're sending transactions, use Ledger as a signer instead \
        of using this command.",
    ));

    ui.print_notification("Please confirm the signing operation on your Ledger".to_string());

    let signature = ledger
        .sign_hash(args.path.clone(), &hash)
        .await
        .context("Failed to sign hash with Ledger")?;

    Ok(LedgerResponse::Signature(SignatureResponse {
        signature: format!("0x{signature}"),
    }))
}

async fn app_version<T: LedgerAsync + 'static>(
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

pub async fn ledger(ledger_args: &Ledger, ui: &UI) -> Result<LedgerResponse> {
    let ledger = create_ledger_app().await?;

    match &ledger_args.subcommand {
        LedgerSubcommand::GetPublicKey(args) => get_public_key(args, ledger, ui).await,
        LedgerSubcommand::SignHash(args) => sign_hash(args, ledger, ui).await,
        LedgerSubcommand::AppVersion(args) => app_version(args, ledger).await,
    }
}
