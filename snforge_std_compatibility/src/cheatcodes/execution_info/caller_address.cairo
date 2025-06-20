use super::{
    CheatArguments, CheatSpan, ContractAddress, ExecutionInfoMock, Operation, cheat_execution_info,
};

/// Changes the caller address for the given contract address and span.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to cheat
/// - `caller_address` - caller address to be set
/// - `span` - instance of `CheatSpan` specifying the number of contract calls with the cheat
/// applied
pub fn cheat_caller_address(
    contract_address: ContractAddress, caller_address: ContractAddress, span: CheatSpan,
) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .caller_address =
            Operation::Start(
                CheatArguments { value: caller_address, span, target: contract_address },
            );

    cheat_execution_info(execution_info);
}

/// Changes the caller address.
/// - `caller_address` - caller address to be set
pub fn start_cheat_caller_address_global(caller_address: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.caller_address = Operation::StartGlobal(caller_address);

    cheat_execution_info(execution_info);
}

/// Cancels the `start_cheat_caller_address_global`.
pub fn stop_cheat_caller_address_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.caller_address = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the caller address for the given contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to cheat
/// - `caller_address` - caller address to be set
pub fn start_cheat_caller_address(
    contract_address: ContractAddress, caller_address: ContractAddress,
) {
    cheat_caller_address(contract_address, caller_address, CheatSpan::Indefinite);
}

/// Cancels the `cheat_caller_address` / `start_cheat_caller_address` for the given
/// contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to stop cheating
pub fn stop_cheat_caller_address(contract_address: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.caller_address = Operation::Stop(contract_address);

    cheat_execution_info(execution_info);
}
