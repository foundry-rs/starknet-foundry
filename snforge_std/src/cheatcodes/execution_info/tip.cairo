use super::{
    CheatArguments, CheatSpan, ContractAddress, ExecutionInfoMock, Operation, cheat_execution_info,
};

/// Changes the transaction tip for the given contract address and span.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to cheat
/// - `tip` - transaction tip to be set
/// - `span` - instance of `CheatSpan` specifying the number of contract calls with the cheat
/// applied
pub fn cheat_tip(contract_address: ContractAddress, tip: u128, span: CheatSpan) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .tx_info
        .tip = Operation::Start(CheatArguments { value: tip, span, target: contract_address });

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

/// Changes the transaction tip for the given contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to cheat
/// - `tip` - transaction tip to be set
pub fn start_cheat_tip(contract_address: ContractAddress, tip: u128) {
    cheat_tip(contract_address, tip, CheatSpan::Indefinite);
}

/// Cancels the `cheat_tip` / `start_cheat_tip` for the given contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to stop cheating
pub fn stop_cheat_tip(contract_address: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.tip = Operation::Stop(contract_address);

    cheat_execution_info(execution_info);
}
