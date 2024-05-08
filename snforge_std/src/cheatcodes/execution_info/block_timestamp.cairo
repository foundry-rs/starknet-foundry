use super::{
    ExecutionInfoMock, Operation, CheatArguments, CheatSpan, cheat_execution_info, ContractAddress
};

/// Changes the block timestamp for the given target and span.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat_block_timestamp
/// - `block_timestamp` - block timestamp to be set
/// - `span` - instance of `CheatSpan` specifying the number of target calls with the cheat applied
fn cheat_block_timestamp(target: ContractAddress, block_timestamp: u64, span: CheatSpan) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .block_info
        .block_timestamp =
            Operation::Start(CheatArguments { value: block_timestamp, span, target, });

    cheat_execution_info(execution_info);
}

/// Changes the block timestamp.
/// - `block_timestamp` - block timestamp to be set
fn cheat_block_timestamp_global(block_timestamp: u64) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.block_timestamp = Operation::StartGlobal(block_timestamp);

    cheat_execution_info(execution_info);
}

/// Cancels the `cheat_block_timestamp_global`
fn stop_cheat_block_timestamp_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.block_timestamp = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the block timestamp for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat
/// - `block_timestamp` - block timestamp to be set
fn start_cheat_block_timestamp(target: ContractAddress, block_timestamp: u64) {
    cheat_block_timestamp(target, block_timestamp, CheatSpan::Indefinite);
}

/// Cancels the `cheat_block_timestamp` / `start_cheat_block_timestamp` for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to stop cheating
fn stop_cheat_block_timestamp(target: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.block_timestamp = Operation::Stop(target);

    cheat_execution_info(execution_info);
}
