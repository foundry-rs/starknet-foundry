use super::{
    ExecutionInfoMock, Operation, CheatArguments, CheatSpan, cheat_execution_info, ContractAddress
};

/// Changes the caller address for the given target and span.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat_caller_address
/// - `caller_address` - caller address to be set
/// - `span` - instance of `CheatSpan` specifying the number of target calls with the cheat applied
fn cheat_caller_address(target: ContractAddress, caller_address: ContractAddress, span: CheatSpan) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .caller_address = Operation::Start(CheatArguments { value: caller_address, span, target, });

    cheat_execution_info(execution_info);
}

/// Changes the caller address.
/// - `caller_address` - caller address to be set
fn cheat_caller_address_global(caller_address: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.caller_address = Operation::StartGlobal(caller_address);

    cheat_execution_info(execution_info);
}

/// Cancels the `cheat_caller_address_global`
fn stop_cheat_caller_address_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.caller_address = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the caller address for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat
/// - `caller_address` - caller address to be set
fn start_cheat_caller_address(target: ContractAddress, caller_address: ContractAddress) {
    cheat_caller_address(target, caller_address, CheatSpan::Indefinite);
}

/// Cancels the `cheat_caller_address` / `start_cheat_caller_address` for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to stop cheating
fn stop_cheat_caller_address(target: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.caller_address = Operation::Stop(target);

    cheat_execution_info(execution_info);
}
