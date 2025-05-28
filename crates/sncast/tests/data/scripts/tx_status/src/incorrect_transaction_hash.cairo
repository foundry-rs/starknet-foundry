use sncast_std::{ProviderError, ScriptCommandError, StarknetError, tx_status};

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
