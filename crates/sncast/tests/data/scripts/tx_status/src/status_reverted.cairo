use sncast_std::{tx_status, TxStatusResult, ExecutionStatus, FinalityStatus};

fn main() {
    let reverted_tx_hash = 0x00ae35dacba17cde62b8ceb12e3b18f4ab6e103fa2d5e3d9821cb9dc59d59a3c;
    let status = tx_status(reverted_tx_hash).unwrap();

    println!("{}", status);
    println!("{:?}", status);

    assert!(
        TxStatusResult {
            finality_status: FinalityStatus::AcceptedOnL1,
            execution_status: Option::Some(ExecutionStatus::Reverted),
        } == status,
    )
}
