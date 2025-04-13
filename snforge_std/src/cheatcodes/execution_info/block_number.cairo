use super::{
    ExecutionInfoMock, Operation, CheatArguments, CheatSpan, cheat_execution_info, ContractAddress,
};

/// Changes the block number for the given contract address and span.
/// - `target` - instance of `ContractAddress` specifying which contract to cheat
/// - `block_number` - block number to be set
/// - `span` - instance of `CheatSpan` specifying the number of contract calls with the cheat
/// applied
pub fn cheat_block_number(target: ContractAddress, block_number: u64, span: CheatSpan) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .block_info
        .block_number =
            Operation::Start(
                CheatArguments { value: block_number, span, target },
            );

    cheat_execution_info(execution_info);
}

/// Changes the block number.
/// - `block_number` - block number to be set
pub fn start_cheat_block_number_global(block_number: u64) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.block_number = Operation::StartGlobal(block_number);

    cheat_execution_info(execution_info);
}

/// Cancels the `start_cheat_block_number_global`.
pub fn stop_cheat_block_number_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.block_number = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the block number for the given target contract address.
/// - `target contract address` - instance of `ContractAddress` specifying which contract to cheat
/// - `block_number` - block number to be set
pub fn start_cheat_block_number(target: ContractAddress, block_number: u64) {
    cheat_block_number(target, block_number, CheatSpan::Indefinite);
}

/// Cancels the `cheat_block_number` / `start_cheat_block_number` for the given target.
/// - `target` - instance of `ContractAddress` specifying which contract to stop cheating
pub fn stop_cheat_block_number(target: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.block_number = Operation::Stop(target);

    cheat_execution_info(execution_info);
}
