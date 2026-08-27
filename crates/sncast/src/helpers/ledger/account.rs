use super::{SncastLedgerTransport, create_ledger_app};
use crate::response::ui::UIExt;
use foundry_ui::UI;
use starknet_rust::signers::{DerivationPath, LedgerError as StarknetLedgerError, LedgerSigner};
use starknet_types_core::felt::Felt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LedgerError {
    #[error(
        "Failed to connect to the Ledger Starknet app. Ensure the device is connected and unlocked and the Starknet app is open"
    )]
    Connect {
        #[source]
        source: StarknetLedgerError,
    },

    #[error(
        "Failed to create Ledger signer. Ensure the derivation path is correct and the Ledger app is ready"
    )]
    Create {
        #[source]
        source: StarknetLedgerError,
    },

    #[error(
        "Public key mismatch!\n\
        Ledger public key: {ledger_public_key:#x}\n\
        Stored public key: {stored_public_key:#x}\n\
        \n\
        This account was created with a different Ledger derivation path or public key.\n\
        Make sure you're using the same derivation path that was used during account creation."
    )]
    PublicKeyMismatch {
        ledger_public_key: Felt,
        stored_public_key: Felt,
    },

    #[error("Failed to get public key from Ledger")]
    PublicKeyFetch {
        #[source]
        source: StarknetLedgerError,
    },
}

pub async fn create_ledger_signer(
    ledger_path: &DerivationPath,
    ui: &UI,
    print_message: bool,
) -> Result<LedgerSigner<SncastLedgerTransport>, LedgerError> {
    let ledger_app = create_ledger_app()
        .await
        .map_err(|source| LedgerError::Connect { source })?;

    if print_message {
        ui.print_notification(
            "Ledger device will display a confirmation screen. Please approve it to continue...\n",
        );
    }

    LedgerSigner::new_with_app(ledger_path.clone(), ledger_app)
        .map_err(|source| LedgerError::Create { source })
}

pub fn verify_ledger_public_key(
    ledger_public_key: Felt,
    stored_public_key: Felt,
) -> Result<(), LedgerError> {
    (ledger_public_key == stored_public_key).ok_or_else(|| LedgerError::PublicKeyMismatch {
        ledger_public_key,
        stored_public_key,
    })
}

pub async fn get_ledger_public_key(
    ledger_path: &DerivationPath,
    ui: &UI,
) -> Result<Felt, LedgerError> {
    let ledger_app = create_ledger_app()
        .await
        .map_err(|source| LedgerError::Connect { source })?;

    ui.print_notification("Please confirm the public key on your Ledger device...\n");

    let public_key = ledger_app
        .get_public_key(ledger_path.clone(), true)
        .await
        .map_err(|source| LedgerError::PublicKeyFetch { source })?;

    Ok(public_key.scalar())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as _;

    #[test]
    fn connection_failure_does_not_blame_derivation_path_and_keeps_source() {
        let error = LedgerError::Connect {
            source: StarknetLedgerError::TransportError(coins_ledger::LedgerError::BackendGone),
        };

        let message = error.to_string();
        assert!(message.contains("Failed to connect to the Ledger Starknet app"));
        assert!(!message.contains("derivation path"));
        assert_eq!(
            error.source().map(ToString::to_string).as_deref(),
            Some("The backend has been disconnected.")
        );
    }
}
