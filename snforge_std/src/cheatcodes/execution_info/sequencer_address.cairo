use super::{
    ExecutionInfoMock, Operation, CheatArguments, CheatSpan, cheat_execution_info, ContractAddress
};

/// Changes the sequencer address for the given target and span.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat_sequencer_address
/// - `sequencer_address` - sequencer address to be set
/// - `span` - instance of `CheatSpan` specifying the number of target calls with the cheat applied
fn cheat_sequencer_address(
    target: ContractAddress, sequencer_address: ContractAddress, span: CheatSpan
) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .block_info
        .sequencer_address =
            Operation::Start(CheatArguments { value: sequencer_address, span, target, });

    cheat_execution_info(execution_info);
}

/// Changes the sequencer address.
/// - `sequencer_address` - sequencer address to be set
fn cheat_sequencer_address_global(sequencer_address: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.sequencer_address = Operation::StartGlobal(sequencer_address);

    cheat_execution_info(execution_info);
}

/// Cancels the `cheat_sequencer_address_global`
fn stop_cheat_sequencer_address_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.sequencer_address = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the sequencer address for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat
/// - `sequencer_address` - sequencer address to be set
fn start_cheat_sequencer_address(target: ContractAddress, sequencer_address: ContractAddress) {
    cheat_sequencer_address(target, sequencer_address, CheatSpan::Indefinite);
}

/// Cancels the `cheat_sequencer_address` / `start_cheat_sequencer_address` for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to stop cheating
fn stop_cheat_sequencer_address(target: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.sequencer_address = Operation::Stop(target);

    cheat_execution_info(execution_info);
}
