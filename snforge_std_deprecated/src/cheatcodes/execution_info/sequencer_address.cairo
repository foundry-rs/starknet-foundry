use super::{
    ExecutionInfoMock, Operation, CheatArguments, CheatSpan, cheat_execution_info, ContractAddress,
};

/// Changes the sequencer address for the given contract address and span.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to cheat
/// - `sequencer_address` - sequencer address to be set
/// - `span` - instance of `CheatSpan` specifying the number of contract calls with the cheat
/// applied
pub fn cheat_sequencer_address(
    contract_address: ContractAddress, sequencer_address: ContractAddress, span: CheatSpan,
) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .block_info
        .sequencer_address =
            Operation::Start(
                CheatArguments { value: sequencer_address, span, target: contract_address },
            );

    cheat_execution_info(execution_info);
}

/// Changes the sequencer address.
/// - `sequencer_address` - sequencer address to be set
pub fn start_cheat_sequencer_address_global(sequencer_address: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.sequencer_address = Operation::StartGlobal(sequencer_address);

    cheat_execution_info(execution_info);
}

/// Cancels the `start_cheat_sequencer_address_global`.
pub fn stop_cheat_sequencer_address_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.sequencer_address = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the sequencer address for the given contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to cheat
/// - `sequencer_address` - sequencer address to be set
pub fn start_cheat_sequencer_address(
    contract_address: ContractAddress, sequencer_address: ContractAddress,
) {
    cheat_sequencer_address(contract_address, sequencer_address, CheatSpan::Indefinite);
}

/// Cancels the `cheat_sequencer_address` / `start_cheat_sequencer_address` for the given
/// contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to stop cheating
pub fn stop_cheat_sequencer_address(contract_address: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.sequencer_address = Operation::Stop(contract_address);

    cheat_execution_info(execution_info);
}
