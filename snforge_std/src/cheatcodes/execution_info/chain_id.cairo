use super::{
    ExecutionInfoMock, Operation, CheatArguments, CheatSpan, cheat_execution_info, ContractAddress
};

/// Changes the transaction chain_id for the given target and span.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat_chain_id
/// - `chain_id` - transaction chain_id to be set
/// - `span` - instance of `CheatSpan` specifying the number of target calls with the cheat applied
fn cheat_chain_id(target: ContractAddress, chain_id: felt252, span: CheatSpan) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .tx_info
        .chain_id = Operation::Start(CheatArguments { value: chain_id, span, target, });

    cheat_execution_info(execution_info);
}

/// Changes the transaction chain_id.
/// - `chain_id` - transaction chain_id to be set
fn cheat_chain_id_global(chain_id: felt252) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.chain_id = Operation::StartGlobal(chain_id);

    cheat_execution_info(execution_info);
}

/// Cancels the `cheat_chain_id_global`
fn stop_cheat_chain_id_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.chain_id = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the transaction chain_id for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat
/// - `chain_id` - transaction chain_id to be set
fn start_cheat_chain_id(target: ContractAddress, chain_id: felt252) {
    cheat_chain_id(target, chain_id, CheatSpan::Indefinite);
}

/// Cancels the `cheat_chain_id` / `start_cheat_chain_id` for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to stop cheating
fn stop_cheat_chain_id(target: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.chain_id = Operation::Stop(target);

    cheat_execution_info(execution_info);
}
