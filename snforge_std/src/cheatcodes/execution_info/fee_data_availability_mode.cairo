use super::{
    ExecutionInfoMock, Operation, CheatArguments, CheatSpan, cheat_execution_info, ContractAddress
};

/// Changes the transaction fee data availability mode for the given target and span.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat_fee_data_availability_mode
/// - `fee_data_availability_mode` - transaction fee data availability mode to be set
/// - `span` - instance of `CheatSpan` specifying the number of target calls with the cheat applied
fn cheat_fee_data_availability_mode(
    target: ContractAddress, fee_data_availability_mode: u32, span: CheatSpan
) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .tx_info
        .fee_data_availability_mode =
            Operation::Start(CheatArguments { value: fee_data_availability_mode, span, target, });

    cheat_execution_info(execution_info);
}

/// Changes the transaction fee data availability mode.
/// - `fee_data_availability_mode` - transaction fee data availability mode to be set
fn cheat_fee_data_availability_mode_global(fee_data_availability_mode: u32) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .tx_info
        .fee_data_availability_mode = Operation::StartGlobal(fee_data_availability_mode);

    cheat_execution_info(execution_info);
}

/// Cancels the `cheat_fee_data_availability_mode_global`
fn stop_cheat_fee_data_availability_mode_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.fee_data_availability_mode = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the transaction fee data availability mode for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat
/// - `fee_data_availability_mode` - transaction fee data availability mode to be set
fn start_cheat_fee_data_availability_mode(
    target: ContractAddress, fee_data_availability_mode: u32
) {
    cheat_fee_data_availability_mode(target, fee_data_availability_mode, CheatSpan::Indefinite);
}

/// Cancels the `cheat_fee_data_availability_mode` / `start_cheat_fee_data_availability_mode` for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to stop cheating
fn stop_cheat_fee_data_availability_mode(target: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.fee_data_availability_mode = Operation::Stop(target);

    cheat_execution_info(execution_info);
}
