use super::{
    ExecutionInfoMock, Operation, CheatArguments, CheatSpan, cheat_execution_info, ContractAddress,
};

/// Changes the transaction nonce data availability mode for the given contract address and span.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat
/// - `nonce_data_availability_mode` - transaction nonce data availability mode to be set
/// - `span` - instance of `CheatSpan` specifying the number of contract calls with the cheat
/// applied
pub fn cheat_nonce_data_availability_mode(
    target: ContractAddress, nonce_data_availability_mode: u32, span: CheatSpan,
) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .tx_info
        .nonce_data_availability_mode =
            Operation::Start(
                CheatArguments {
                    value: nonce_data_availability_mode, span, target,
                },
            );

    cheat_execution_info(execution_info);
}

/// Changes the transaction nonce data availability mode.
/// - `nonce_data_availability_mode` - transaction nonce data availability mode to be set
pub fn start_cheat_nonce_data_availability_mode_global(nonce_data_availability_mode: u32) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .tx_info
        .nonce_data_availability_mode = Operation::StartGlobal(nonce_data_availability_mode);

    cheat_execution_info(execution_info);
}

/// Cancels the `start_cheat_nonce_data_availability_mode_global`.
pub fn stop_cheat_nonce_data_availability_mode_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.nonce_data_availability_mode = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the transaction nonce data availability mode for the given target contract address.
/// - `target` - instance of `ContractAddress` specifying which contract to cheat
/// - `nonce_data_availability_mode` - transaction nonce data availability mode to be set
pub fn start_cheat_nonce_data_availability_mode(
    target: ContractAddress, nonce_data_availability_mode: u32,
) {
    cheat_nonce_data_availability_mode(
        target, nonce_data_availability_mode, CheatSpan::Indefinite,
    );
}

/// Cancels the `cheat_nonce_data_availability_mode` / `start_cheat_nonce_data_availability_mode`
/// for the given target contract address.
/// - `target` - instance of `ContractAddress` specifying which contract to stop cheating
pub fn stop_cheat_nonce_data_availability_mode(target: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.nonce_data_availability_mode = Operation::Stop(target);

    cheat_execution_info(execution_info);
}
