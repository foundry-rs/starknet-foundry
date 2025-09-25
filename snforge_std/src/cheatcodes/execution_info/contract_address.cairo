//! These cheatcodes are currently used only internally by the `interact_with_state` cheatcode,
//! and are specifically intended to cheat on the `TEST_ADDRESS`.
//! They are not exposed through the public API in a general form, as there are no known use cases
//! that would require them.

use crate::test_address;
use super::{
    CheatArguments, CheatSpan, ContractAddress, ExecutionInfoMock, Operation, cheat_execution_info,
};

/// Overrides the contract address for the default test address.
/// After this function is called, any call to `starknet::get_contract_address()`
/// within the test context will return the provided `contract_address`.
/// - `contract_address` - The contract address to use for the default test address.
pub(crate) fn start_cheat_contract_address(contract_address: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .contract_address =
            Operation::Start(
                CheatArguments {
                    value: contract_address, span: CheatSpan::Indefinite, target: test_address(),
                },
            );

    cheat_execution_info(execution_info);
}

/// Cancels the `start_cheat_contract_address`.
pub(crate) fn stop_cheat_contract_address() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.contract_address = Operation::Stop(test_address());

    cheat_execution_info(execution_info);
}
