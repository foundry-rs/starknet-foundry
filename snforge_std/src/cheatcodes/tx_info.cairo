use starknet::{ContractAddress, testing::cheatcode, contract_address_const};
use starknet::info::v2::ResourceBounds;
use snforge_std::cheatcodes::CheatSpan;
use super::super::_cheatcode::handle_cheatcode;
use super::execution_info::{cheat_execution_info, Operation, ExecutionInfoMock, TxInfoMock};


/// Changes `TxInfo` returned by `get_tx_info()` for the targeted contract and span.
/// - `tx_info_mock` - a struct with same structure as `TxInfo` (returned by `get_tx_info()`)
fn spoof(tx_info_mock: TxInfoMock) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.tx_info = tx_info_mock;

    cheat_execution_info(execution_info);
}

/// Changes `TxInfo` returned by `get_tx_info()` for the targeted contract until the spoof is canceled with `stop_spoof`.
/// - `tx_info_mock` - a struct with same structure as `TxInfo` (returned by `get_tx_info()`)
fn start_spoof(tx_info_mock: TxInfoMock) {
    spoof(tx_info_mock);
}

/// Cancels the `spoof` for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to stop spoofing
fn stop_spoof(target: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .tx_info =
            TxInfoMock {
                version: Operation::Stop(target),
                account_contract_address: Operation::Stop(target),
                max_fee: Operation::Stop(target),
                signature: Operation::Stop(target),
                transaction_hash: Operation::Stop(target),
                chain_id: Operation::Stop(target),
                nonce: Operation::Stop(target),
                resource_bounds: Operation::Stop(target),
                tip: Operation::Stop(target),
                paymaster_data: Operation::Stop(target),
                nonce_data_availability_mode: Operation::Stop(target),
                fee_data_availability_mode: Operation::Stop(target),
                account_deployment_data: Operation::Stop(target),
            };

    cheat_execution_info(execution_info);
}

