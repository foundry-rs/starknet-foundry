use super::{
    ExecutionInfoMock, Operation, CheatArguments, CheatSpan, cheat_execution_info, ContractAddress
};

/// Changes the transaction nonce for the given target and span.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat_nonce
/// - `nonce` - transaction nonce to be set
/// - `span` - instance of `CheatSpan` specifying the number of target calls with the cheat applied
fn cheat_nonce(target: ContractAddress, nonce: felt252, span: CheatSpan) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.nonce = Operation::Start(CheatArguments { value: nonce, span, target, });

    cheat_execution_info(execution_info);
}

/// Changes the transaction nonce.
/// - `nonce` - transaction nonce to be set
fn cheat_nonce_global(nonce: felt252) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.nonce = Operation::StartGlobal(nonce);

    cheat_execution_info(execution_info);
}

/// Cancels the `cheat_nonce_global`
fn stop_cheat_nonce_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.nonce = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the transaction nonce for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat
/// - `nonce` - transaction nonce to be set
fn start_cheat_nonce(target: ContractAddress, nonce: felt252) {
    cheat_nonce(target, nonce, CheatSpan::Indefinite);
}

/// Cancels the `cheat_nonce` / `start_cheat_nonce` for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to stop cheating
fn stop_cheat_nonce(target: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.nonce = Operation::Stop(target);

    cheat_execution_info(execution_info);
}
