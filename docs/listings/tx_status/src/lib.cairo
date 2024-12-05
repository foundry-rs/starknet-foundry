use sncast_std::{tx_status};

fn main() {
    let transaction_hash = 0x00ae35dacba17cde62b8ceb12e3b18f4ab6e103fa2d5e3d9821cb9dc59d59a3c;
    let status = tx_status(transaction_hash).expect('Failed to get status');

    println!("transaction status: {:?}", status);
}
