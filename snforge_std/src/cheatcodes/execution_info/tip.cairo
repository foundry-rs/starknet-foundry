use super::{
    ExecutionInfoMock, Operation, CheatArguments, CheatSpan, cheat_execution_info, ContractAddress,
};

/// Changes the transaction tip for the given contract address and span.
/// - `target` - instance of `ContractAddress` specifying which contract to cheat
/// - `tip` - transaction tip to be set
/// - `span` - instance of `CheatSpan` specifying the number of contract calls with the cheat
/// applied
pub fn cheat_tip(target: ContractAddress, tip: u128, span: CheatSpan) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .tx_info
        .tip = Operation::Start(CheatArguments { value: tip, span, target });

    cheat_execution_info(execution_info);
}

/// Changes the transaction tip.
/// - `tip` - transaction tip to be set
pub fn start_cheat_tip_global(tip: u128) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.tip = Operation::StartGlobal(tip);

    cheat_execution_info(execution_info);
}

/// Cancels the `start_cheat_tip_global`.
pub fn stop_cheat_tip_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.tip = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the transaction tip for the given target contract address.
/// - `target` - instance of `ContractAddress` specifying which contract to cheat
/// - `tip` - transaction tip to be set
pub fn start_cheat_tip(target: ContractAddress, tip: u128) {
    cheat_tip(target, tip, CheatSpan::Indefinite);
}

/// Cancels the `cheat_tip` / `start_cheat_tip` for the given target contract address.
/// - `target` - instance of `ContractAddress` specifying which contract to stop cheating
pub fn stop_cheat_tip(target: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.tip = Operation::Stop(target);

    cheat_execution_info(execution_info);
}
