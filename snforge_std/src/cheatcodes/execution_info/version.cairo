use super::{
    ExecutionInfoMock, Operation, CheatArguments, CheatSpan, cheat_execution_info, ContractAddress
};

/// Changes the transaction version for the given contract address and span.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to cheat
/// - `version` - transaction version to be set
/// - `span` - instance of `CheatSpan` specifying the number of contract calls with the cheat
/// applied
pub fn cheat_transaction_version(contract_address: ContractAddress, version: felt252, span: CheatSpan) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .tx_info
        .version =
            Operation::Start(CheatArguments { value: version, span, target: contract_address, });

    cheat_execution_info(execution_info);
}

/// Changes the transaction version.
/// - `version` - transaction version to be set
pub fn start_cheat_transaction_version_global(version: felt252) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.version = Operation::StartGlobal(version);

    cheat_execution_info(execution_info);
}

/// Cancels the `start_cheat_transaction_version_global`.
pub fn stop_cheat_transaction_version_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.version = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the transaction version for the given contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to cheat
/// - `version` - transaction version to be set
pub fn start_cheat_transaction_version(contract_address: ContractAddress, version: felt252) {
    cheat_transaction_version(contract_address, version, CheatSpan::Indefinite);
}

/// Cancels the `cheat_transaction_version` / `start_cheat_transaction_version` for the given
/// contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to stop cheating
pub fn stop_cheat_transaction_version(contract_address: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.version = Operation::Stop(contract_address);

    cheat_execution_info(execution_info);
}
