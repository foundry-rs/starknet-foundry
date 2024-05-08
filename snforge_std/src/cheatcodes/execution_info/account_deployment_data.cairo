use super::{
    ExecutionInfoMock, Operation, CheatArguments, CheatSpan, cheat_execution_info, ContractAddress
};

/// Changes the transaction account deployment data for the given target and span.
/// - `target` - instance of `ContractAddress` specifying which contracts to
/// cheat_account_deployment_data - `account_deployment_data` - transaction account deployment data
/// to be set - `span` - instance of `CheatSpan` specifying the number of target calls with the
/// cheat applied
fn cheat_account_deployment_data(
    target: ContractAddress, account_deployment_data: Span<felt252>, span: CheatSpan
) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .tx_info
        .account_deployment_data =
            Operation::Start(CheatArguments { value: account_deployment_data, span, target, });

    cheat_execution_info(execution_info);
}

/// Changes the transaction account deployment data.
/// - `account_deployment_data` - transaction account deployment data to be set
fn cheat_account_deployment_data_global(account_deployment_data: Span<felt252>) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .tx_info
        .account_deployment_data = Operation::StartGlobal(account_deployment_data);

    cheat_execution_info(execution_info);
}

/// Cancels the `cheat_account_deployment_data_global`
fn stop_cheat_account_deployment_data_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.account_deployment_data = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the transaction account deployment data for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat
/// - `account_deployment_data` - transaction account deployment data to be set
fn start_cheat_account_deployment_data(
    target: ContractAddress, account_deployment_data: Span<felt252>
) {
    cheat_account_deployment_data(target, account_deployment_data, CheatSpan::Indefinite);
}

/// Cancels the `cheat_account_deployment_data` / `start_cheat_account_deployment_data` for the
/// given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to stop cheating
fn stop_cheat_account_deployment_data(target: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.account_deployment_data = Operation::Stop(target);

    cheat_execution_info(execution_info);
}
