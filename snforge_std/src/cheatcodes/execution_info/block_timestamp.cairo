use super::{
    ExecutionInfoMock, Operation, CheatArguments, CheatSpan, cheat_execution_info, ContractAddress
};

/// Changes the block timestamp for the given contract address and span.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to cheat
/// - `block_timestamp` - block timestamp to be set
/// - `span` - instance of `CheatSpan` specifying the number of contract calls with the cheat
/// applied
pub fn cheat_block_timestamp(contract_address: ContractAddress, block_timestamp: u64, span: CheatSpan) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .block_info
        .block_timestamp =
            Operation::Start(
                CheatArguments { value: block_timestamp, span, target: contract_address, }
            );

    cheat_execution_info(execution_info);
}

/// Changes the block timestamp.
/// - `block_timestamp` - block timestamp to be set
pub fn start_cheat_block_timestamp_global(block_timestamp: u64) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.block_timestamp = Operation::StartGlobal(block_timestamp);

    cheat_execution_info(execution_info);
}

/// Cancels the `start_cheat_block_timestamp_global`.
pub fn stop_cheat_block_timestamp_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.block_timestamp = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the block timestamp for the given contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to cheat
/// - `block_timestamp` - block timestamp to be set
pub fn start_cheat_block_timestamp(contract_address: ContractAddress, block_timestamp: u64) {
    cheat_block_timestamp(contract_address, block_timestamp, CheatSpan::Indefinite);
}

/// Cancels the `cheat_block_timestamp` / `start_cheat_block_timestamp` for the given
/// contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to stop cheating
pub fn stop_cheat_block_timestamp(contract_address: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.block_timestamp = Operation::Stop(contract_address);

    cheat_execution_info(execution_info);
}
