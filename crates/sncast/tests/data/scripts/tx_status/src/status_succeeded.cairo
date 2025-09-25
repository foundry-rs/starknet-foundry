use sncast_std::{ExecutionStatus, FinalityStatus, TxStatusResult, tx_status};

fn main() {
    let succeeded_tx_hash = 0x07d2067cd7675f88493a9d773b456c8d941457ecc2f6201d2fe6b0607daadfd1;
    let status = tx_status(succeeded_tx_hash).unwrap();

    println!("{}", status);
    println!("{:?}", status);

    assert!(
        TxStatusResult {
            finality_status: FinalityStatus::AcceptedOnL1,
            execution_status: Option::Some(ExecutionStatus::Succeeded),
        } == status,
    )
}
