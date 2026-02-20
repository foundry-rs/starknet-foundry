mod hd_path;
mod key_locator;

#[cfg(feature = "ledger-emulator")]
mod emulator_transport;

pub use hd_path::{DerivationPathParser, ParsedDerivationPath};
pub use key_locator::LedgerKeyLocator;

use starknet_rust::signers::ledger::LedgerStarknetApp;

#[cfg(feature = "ledger-emulator")]
pub type SncastLedgerTransport = emulator_transport::SpeculosHttpTransport;

#[cfg(not(feature = "ledger-emulator"))]
pub type SncastLedgerTransport = coins_ledger::transports::Ledger;

pub async fn create_ledger_app() -> anyhow::Result<LedgerStarknetApp<SncastLedgerTransport>> {
    #[cfg(feature = "ledger-emulator")]
    let app = emulator_transport::emulator_ledger_app().await?;
    #[cfg(not(feature = "ledger-emulator"))]
    let app = LedgerStarknetApp::new().await?;
    Ok(app)
}
