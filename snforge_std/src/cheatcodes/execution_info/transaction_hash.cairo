use super::{
    ExecutionInfoMock, Operation, CheatArguments, CheatSpan, cheat_execution_info, ContractAddress
};

/// Changes the transaction hash for the given target and span.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat_transaction_hash
/// - `transaction_hash` - transaction hash to be set
/// - `span` - instance of `CheatSpan` specifying the number of target calls with the cheat applied
fn cheat_transaction_hash(target: ContractAddress, transaction_hash: felt252, span: CheatSpan) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .tx_info
        .transaction_hash =
            Operation::Start(CheatArguments { value: transaction_hash, span, target, });

    cheat_execution_info(execution_info);
}

/// Changes the transaction hash.
/// - `transaction_hash` - transaction hash to be set
fn cheat_transaction_hash_global(transaction_hash: felt252) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.transaction_hash = Operation::StartGlobal(transaction_hash);

    cheat_execution_info(execution_info);
}

/// Cancels the `cheat_transaction_hash_global`
fn stop_cheat_transaction_hash_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.transaction_hash = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the transaction hash for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat
/// - `transaction_hash` - transaction hash to be set
fn start_cheat_transaction_hash(target: ContractAddress, transaction_hash: felt252) {
    cheat_transaction_hash(target, transaction_hash, CheatSpan::Indefinite);
}

/// Cancels the `cheat_transaction_hash` / `start_cheat_transaction_hash` for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to stop cheating
fn stop_cheat_transaction_hash(target: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.transaction_hash = Operation::Stop(target);

    cheat_execution_info(execution_info);
}
