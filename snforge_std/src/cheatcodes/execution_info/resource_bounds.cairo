use super::{
    ExecutionInfoMock, Operation, CheatArguments, CheatSpan, cheat_execution_info, ContractAddress
};
use starknet::info::v2::ResourceBounds;


/// Changes the transaction resource bounds for the given contract address and span.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to cheat
/// - `resource_bounds` - transaction resource bounds to be set
/// - `span` - instance of `CheatSpan` specifying the number of contract_address calls with the cheat applied
fn cheat_resource_bounds(
    contract_address: ContractAddress, resource_bounds: Span<ResourceBounds>, span: CheatSpan
) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .tx_info
        .resource_bounds =
            Operation::Start(CheatArguments { value: resource_bounds, span, target: contract_address, });

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

/// Changes the transaction resource bounds for the given contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to cheat
/// - `resource_bounds` - transaction resource bounds to be set
fn start_cheat_resource_bounds(contract_address: ContractAddress, resource_bounds: Span<ResourceBounds>) {
    cheat_resource_bounds(contract_address, resource_bounds, CheatSpan::Indefinite);
}

/// Cancels the `cheat_resource_bounds` / `start_cheat_resource_bounds` for the given contract_address.
/// - `contract_address` - instance of `ContractAddress` specifying which contract to stop cheating
fn stop_cheat_resource_bounds(contract_address: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info.resource_bounds = Operation::Stop(contract_address);

    cheat_execution_info(execution_info);
}
