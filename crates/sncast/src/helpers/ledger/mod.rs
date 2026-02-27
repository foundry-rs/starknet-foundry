mod account;
mod hd_path;

#[cfg(feature = "ledger-emulator")]
mod emulator_transport;

pub use account::{create_ledger_signer, get_ledger_public_key, ledger_account};
pub use hd_path::{DerivationPathParser, parse_derivation_path};
use starknet_rust::signers::ledger::LedgerStarknetApp;

#[cfg(feature = "ledger-emulator")]
pub type SncastLedgerTransport = emulator_transport::SpeculosHttpTransport;

#[cfg(not(feature = "ledger-emulator"))]
pub type SncastLedgerTransport = coins_ledger::transports::Ledger;

#[allow(clippy::unused_async)]
pub async fn create_ledger_app() -> anyhow::Result<LedgerStarknetApp<SncastLedgerTransport>> {
    #[cfg(feature = "ledger-emulator")]
    let app = emulator_transport::emulator_ledger_app()?;
    #[cfg(not(feature = "ledger-emulator"))]
    let app = LedgerStarknetApp::new().await?;
    Ok(app)
}
