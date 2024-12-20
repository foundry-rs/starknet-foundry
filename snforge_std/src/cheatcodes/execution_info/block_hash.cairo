use super::{
    ExecutionInfoMock, Operation, CheatArguments, CheatSpan, cheat_execution_info, ContractAddress
};

/// Changes the block hash for the given contract address and span.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to cheat
/// - `block_hash` - block hash to be set
/// - `span` - instance of `CheatSpan` specifying the hash of contract calls with the cheat
/// applied
pub fn cheat_block_hash(contract_address: ContractAddress, block_hash: felt252, span: CheatSpan) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .block_info
        .block_hash =
            Operation::Start(CheatArguments { value: block_hash, span, target: contract_address, });

    cheat_execution_info(execution_info);
}

/// Changes the block hash.
/// - `block_hash` - block hash to be set
pub fn start_cheat_block_hash_global(block_hash: felt252) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.block_hash = Operation::StartGlobal(block_hash);

    cheat_execution_info(execution_info);
}

/// Cancels the `start_cheat_block_hash_global`.
pub fn stop_cheat_block_hash_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.block_hash = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the block hash for the given contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to cheat
/// - `block_hash` - block hash to be set
pub fn start_cheat_block_hash(contract_address: ContractAddress, block_hash: felt252) {
    cheat_block_hash(contract_address, block_hash, CheatSpan::Indefinite);
}

/// Cancels the `cheat_block_hash` / `start_cheat_block_hash` for the given contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to stop cheating
pub fn stop_cheat_block_hash(contract_address: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.block_hash = Operation::Stop(contract_address);

    cheat_execution_info(execution_info);
}
