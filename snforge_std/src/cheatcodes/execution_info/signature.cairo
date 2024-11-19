use super::{
    ExecutionInfoMock, Operation, CheatArguments, CheatSpan, cheat_execution_info, ContractAddress
};

/// Changes the transaction signature for the given contract address and span.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to cheat
/// - `signature` - transaction signature to be set
/// - `span` - instance of `CheatSpan` specifying the number of contract calls with the cheat
/// applied
pub fn cheat_signature(contract_address: ContractAddress, signature: Span<felt252>, span: CheatSpan) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .tx_info
        .signature =
            Operation::Start(CheatArguments { value: signature, span, target: contract_address, });

    cheat_execution_info(execution_info);
}

/// Changes the transaction signature.
/// - `signature` - transaction signature to be set
pub fn start_cheat_signature_global(signature: Span<felt252>) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.signature = Operation::StartGlobal(signature);

    cheat_execution_info(execution_info);
}

/// Cancels the `start_cheat_signature_global`.
pub fn stop_cheat_signature_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.signature = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the transaction signature for the given contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to cheat
/// - `signature` - transaction signature to be set
pub fn start_cheat_signature(contract_address: ContractAddress, signature: Span<felt252>) {
    cheat_signature(contract_address, signature, CheatSpan::Indefinite);
}

/// Cancels the `cheat_signature` / `start_cheat_signature` for the given contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to stop cheating
pub fn stop_cheat_signature(contract_address: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.signature = Operation::Stop(contract_address);

    cheat_execution_info(execution_info);
}
