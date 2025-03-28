use super::{
    ExecutionInfoMock, Operation, CheatArguments, CheatSpan, cheat_execution_info, ContractAddress,
};

/// Changes the transaction max fee for the given contract address and span.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to cheat
/// - `max_fee` - transaction max fee to be set
/// - `span` - instance of `CheatSpan` specifying the number of contract calls with the cheat
/// applied
pub fn cheat_max_fee(contract_address: ContractAddress, max_fee: u128, span: CheatSpan) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .tx_info
        .max_fee =
            Operation::Start(CheatArguments { value: max_fee, span, target: contract_address });

    cheat_execution_info(execution_info);
}

/// Changes the transaction max fee.
/// - `max_fee` - transaction max fee to be set
pub fn start_cheat_max_fee_global(max_fee: u128) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.max_fee = Operation::StartGlobal(max_fee);

    cheat_execution_info(execution_info);
}

/// Cancels the `start_cheat_max_fee_global`.
pub fn stop_cheat_max_fee_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.max_fee = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the transaction max fee for the given contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to cheat
/// - `max_fee` - transaction max fee to be set
pub fn start_cheat_max_fee(contract_address: ContractAddress, max_fee: u128) {
    cheat_max_fee(contract_address, max_fee, CheatSpan::Indefinite);
}

/// Cancels the `cheat_max_fee` / `start_cheat_max_fee` for the given contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to stop cheating
pub fn stop_cheat_max_fee(contract_address: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.max_fee = Operation::Stop(contract_address);

    cheat_execution_info(execution_info);
}
