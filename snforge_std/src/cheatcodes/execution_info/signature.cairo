use super::{
    ExecutionInfoMock, Operation, CheatArguments, CheatSpan, cheat_execution_info, ContractAddress
};

/// Changes the transaction signature for the given target and span.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat_signature
/// - `signature` - transaction signature to be set
/// - `span` - instance of `CheatSpan` specifying the number of target calls with the cheat applied
fn cheat_signature(target: ContractAddress, signature: Span<felt252>, span: CheatSpan) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .tx_info
        .signature = Operation::Start(CheatArguments { value: signature, span, target, });

    cheat_execution_info(execution_info);
}

/// Changes the transaction signature.
/// - `signature` - transaction signature to be set
fn cheat_signature_global(signature: Span<felt252>) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.signature = Operation::StartGlobal(signature);

    cheat_execution_info(execution_info);
}

/// Cancels the `cheat_signature_global`
fn stop_cheat_signature_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.signature = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the transaction signature for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat
/// - `signature` - transaction signature to be set
fn start_cheat_signature(target: ContractAddress, signature: Span<felt252>) {
    cheat_signature(target, signature, CheatSpan::Indefinite);
}

/// Cancels the `cheat_signature` / `start_cheat_signature` for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to stop cheating
fn stop_cheat_signature(target: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.signature = Operation::Stop(target);

    cheat_execution_info(execution_info);
}
