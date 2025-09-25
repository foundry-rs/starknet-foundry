use super::{
    CheatArguments, CheatSpan, ContractAddress, ExecutionInfoMock, Operation, cheat_execution_info,
};

/// Changes the transaction chain_id for the given contract address and span.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to cheat
/// - `chain_id` - transaction chain_id to be set
/// - `span` - instance of `CheatSpan` specifying the number of contract calls with the cheat
/// applied
pub fn cheat_chain_id(contract_address: ContractAddress, chain_id: felt252, span: CheatSpan) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .tx_info
        .chain_id =
            Operation::Start(CheatArguments { value: chain_id, span, target: contract_address });

    cheat_execution_info(execution_info);
}

/// Changes the transaction chain_id.
/// - `chain_id` - transaction chain_id to be set
pub fn start_cheat_chain_id_global(chain_id: felt252) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.chain_id = Operation::StartGlobal(chain_id);

    cheat_execution_info(execution_info);
}

/// Cancels the `start_cheat_chain_id_global`.
pub fn stop_cheat_chain_id_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.chain_id = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the transaction chain_id for the given contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to cheat
/// - `chain_id` - transaction chain_id to be set
pub fn start_cheat_chain_id(contract_address: ContractAddress, chain_id: felt252) {
    cheat_chain_id(contract_address, chain_id, CheatSpan::Indefinite);
}

/// Cancels the `cheat_chain_id` / `start_cheat_chain_id` for the given contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to stop cheating
pub fn stop_cheat_chain_id(contract_address: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.chain_id = Operation::Stop(contract_address);

    cheat_execution_info(execution_info);
}
