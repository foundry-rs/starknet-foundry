use super::{
    ExecutionInfoMock, Operation, CheatArguments, CheatSpan, cheat_execution_info, ContractAddress
};

/// Changes the transaction nonce for the given contract address and span.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to cheat
/// - `nonce` - transaction nonce to be set
/// - `span` - instance of `CheatSpan` specifying the number of contract calls with the cheat
/// applied
pub fn cheat_nonce(contract_address: ContractAddress, nonce: felt252, span: CheatSpan) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .tx_info
        .nonce = Operation::Start(CheatArguments { value: nonce, span, target: contract_address, });

    cheat_execution_info(execution_info);
}

/// Changes the transaction nonce.
/// - `nonce` - transaction nonce to be set
pub fn start_cheat_nonce_global(nonce: felt252) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.nonce = Operation::StartGlobal(nonce);

    cheat_execution_info(execution_info);
}

/// Cancels the `start_cheat_nonce_global`.
pub fn stop_cheat_nonce_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.nonce = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the transaction nonce for the given contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to cheat
/// - `nonce` - transaction nonce to be set
pub fn start_cheat_nonce(contract_address: ContractAddress, nonce: felt252) {
    cheat_nonce(contract_address, nonce, CheatSpan::Indefinite);
}

/// Cancels the `cheat_nonce` / `start_cheat_nonce` for the given contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to stop cheating
pub fn stop_cheat_nonce(contract_address: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.nonce = Operation::Stop(contract_address);

    cheat_execution_info(execution_info);
}
