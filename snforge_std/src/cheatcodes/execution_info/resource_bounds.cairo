use super::{
    ExecutionInfoMock, Operation, CheatArguments, CheatSpan, cheat_execution_info, ContractAddress
};
use starknet::info::v2::ResourceBounds;


/// Changes the transaction resource bounds for the given target and span.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat_resource_bounds
/// - `resource_bounds` - transaction resource bounds to be set
/// - `span` - instance of `CheatSpan` specifying the number of target calls with the cheat applied
fn cheat_resource_bounds(
    target: ContractAddress, resource_bounds: Span<ResourceBounds>, span: CheatSpan
) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .tx_info
        .resource_bounds =
            Operation::Start(CheatArguments { value: resource_bounds, span, target, });

    cheat_execution_info(execution_info);
}

/// Changes the transaction resource bounds.
/// - `resource_bounds` - transaction resource bounds to be set
fn cheat_resource_bounds_global(resource_bounds: Span<ResourceBounds>) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.resource_bounds = Operation::StartGlobal(resource_bounds);

    cheat_execution_info(execution_info);
}

/// Cancels the `cheat_resource_bounds_global`
fn stop_cheat_resource_bounds_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.resource_bounds = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the transaction resource bounds for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat
/// - `resource_bounds` - transaction resource bounds to be set
fn start_cheat_resource_bounds(target: ContractAddress, resource_bounds: Span<ResourceBounds>) {
    cheat_resource_bounds(target, resource_bounds, CheatSpan::Indefinite);
}

/// Cancels the `cheat_resource_bounds` / `start_cheat_resource_bounds` for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to stop cheating
fn stop_cheat_resource_bounds(target: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.resource_bounds = Operation::Stop(target);

    cheat_execution_info(execution_info);
}
