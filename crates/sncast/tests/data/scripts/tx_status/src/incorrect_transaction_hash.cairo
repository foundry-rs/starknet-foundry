use sncast_std::{tx_status, ScriptCommandError, ProviderError, StarknetError};

fn main() {
    let incorrect_tx_hash = 0x1;
    let status = tx_status(incorrect_tx_hash).unwrap_err();
    println!("{:?}", status);

    assert!(
        ScriptCommandError::ProviderError(
            ProviderError::StarknetError(StarknetError::TransactionHashNotFound(())),
        ) == status,
    )
}
