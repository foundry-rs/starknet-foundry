use super::{
    ExecutionInfoMock, Operation, CheatArguments, CheatSpan, cheat_execution_info, ContractAddress
};

/// Changes the transaction tip for the given target and span.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat_tip
/// - `tip` - transaction tip to be set
/// - `span` - instance of `CheatSpan` specifying the number of target calls with the cheat applied
fn cheat_tip(target: ContractAddress, tip: u128, span: CheatSpan) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.tip = Operation::Start(CheatArguments { value: tip, span, target, });

    cheat_execution_info(execution_info);
}

/// Changes the transaction tip.
/// - `tip` - transaction tip to be set
fn cheat_tip_global(tip: u128) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.tip = Operation::StartGlobal(tip);

    cheat_execution_info(execution_info);
}

/// Cancels the `cheat_tip_global`
fn stop_cheat_tip_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.tip = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the transaction tip for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat
/// - `tip` - transaction tip to be set
fn start_cheat_tip(target: ContractAddress, tip: u128) {
    cheat_tip(target, tip, CheatSpan::Indefinite);
}

/// Cancels the `cheat_tip` / `start_cheat_tip` for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to stop cheating
fn stop_cheat_tip(target: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.tip = Operation::Stop(target);

    cheat_execution_info(execution_info);
}
