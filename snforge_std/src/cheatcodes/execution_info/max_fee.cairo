use super::{
    ExecutionInfoMock, Operation, CheatArguments, CheatSpan, cheat_execution_info, ContractAddress
};

/// Changes the transaction max fee for the given target and span.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat_max_fee
/// - `max_fee` - transaction max fee to be set
/// - `span` - instance of `CheatSpan` specifying the number of target calls with the cheat applied
fn cheat_max_fee(target: ContractAddress, max_fee: u128, span: CheatSpan) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .tx_info
        .max_fee = Operation::Start(CheatArguments { value: max_fee, span, target, });

    cheat_execution_info(execution_info);
}

/// Changes the transaction max fee.
/// - `max_fee` - transaction max fee to be set
fn cheat_max_fee_global(max_fee: u128) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.max_fee = Operation::StartGlobal(max_fee);

    cheat_execution_info(execution_info);
}

/// Cancels the `cheat_max_fee_global`
fn stop_cheat_max_fee_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.max_fee = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the transaction max fee for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat
/// - `max_fee` - transaction max fee to be set
fn start_cheat_max_fee(target: ContractAddress, max_fee: u128) {
    cheat_max_fee(target, max_fee, CheatSpan::Indefinite);
}

/// Cancels the `cheat_max_fee` / `start_cheat_max_fee` for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to stop cheating
fn stop_cheat_max_fee(target: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.max_fee = Operation::Stop(target);

    cheat_execution_info(execution_info);
}
