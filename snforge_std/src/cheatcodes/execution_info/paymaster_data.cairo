use super::{
    ExecutionInfoMock, Operation, CheatArguments, CheatSpan, cheat_execution_info, ContractAddress
};

/// Changes the transaction paymaster data for the given target and span.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat_paymaster_data
/// - `paymaster_data` - transaction paymaster data to be set
/// - `span` - instance of `CheatSpan` specifying the number of target calls with the cheat applied
fn cheat_paymaster_data(target: ContractAddress, paymaster_data: Span<felt252>, span: CheatSpan) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .tx_info
        .paymaster_data = Operation::Start(CheatArguments { value: paymaster_data, span, target, });

    cheat_execution_info(execution_info);
}

/// Changes the transaction paymaster data.
/// - `paymaster_data` - transaction paymaster data to be set
fn cheat_paymaster_data_global(paymaster_data: Span<felt252>) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.paymaster_data = Operation::StartGlobal(paymaster_data);

    cheat_execution_info(execution_info);
}

/// Cancels the `cheat_paymaster_data_global`
fn stop_cheat_paymaster_data_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.paymaster_data = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the transaction paymaster data for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat
/// - `paymaster_data` - transaction paymaster data to be set
fn start_cheat_paymaster_data(target: ContractAddress, paymaster_data: Span<felt252>) {
    cheat_paymaster_data(target, paymaster_data, CheatSpan::Indefinite);
}

/// Cancels the `cheat_paymaster_data` / `start_cheat_paymaster_data` for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to stop cheating
fn stop_cheat_paymaster_data(target: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.paymaster_data = Operation::Stop(target);

    cheat_execution_info(execution_info);
}
