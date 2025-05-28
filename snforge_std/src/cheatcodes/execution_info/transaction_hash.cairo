use super::{
    CheatArguments, CheatSpan, ContractAddress, ExecutionInfoMock, Operation, cheat_execution_info,
};

/// Changes the transaction hash for the given contract address and span.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to cheat
/// - `transaction_hash` - transaction hash to be set
/// - `span` - instance of `CheatSpan` specifying the number of contract calls with the cheat
/// applied
pub fn cheat_transaction_hash(
    contract_address: ContractAddress, transaction_hash: felt252, span: CheatSpan,
) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .tx_info
        .transaction_hash =
            Operation::Start(
                CheatArguments { value: transaction_hash, span, target: contract_address },
            );

    cheat_execution_info(execution_info);
}

/// Changes the transaction hash.
/// - `transaction_hash` - transaction hash to be set
pub fn start_cheat_transaction_hash_global(transaction_hash: felt252) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.transaction_hash = Operation::StartGlobal(transaction_hash);

    cheat_execution_info(execution_info);
}

/// Cancels the `start_cheat_transaction_hash_global`.
pub fn stop_cheat_transaction_hash_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.transaction_hash = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the transaction hash for the given contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to cheat
/// - `transaction_hash` - transaction hash to be set
pub fn start_cheat_transaction_hash(contract_address: ContractAddress, transaction_hash: felt252) {
    cheat_transaction_hash(contract_address, transaction_hash, CheatSpan::Indefinite);
}

/// Cancels the `cheat_transaction_hash` / `start_cheat_transaction_hash` for the given
/// contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to stop cheating
pub fn stop_cheat_transaction_hash(contract_address: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.transaction_hash = Operation::Stop(contract_address);

    cheat_execution_info(execution_info);
}
