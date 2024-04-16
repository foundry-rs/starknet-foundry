use starknet::{ContractAddress, ClassHash, contract_address_const};
use starknet::testing::cheatcode;
use super::_cheatcode::handle_cheatcode;

mod events;
mod l1_handler;
mod contract_class;
mod tx_info;
mod fork;
mod storage;

#[derive(Drop, Serde, PartialEq, Clone, Debug, Display)]
enum CheatTarget {
    All: (),
    One: ContractAddress,
    Multiple: Array<ContractAddress>
}

#[derive(Drop, Serde, PartialEq, Clone, Debug, Display)]
enum CheatSpan {
    Indefinite: (),
    TargetCalls: usize,
}

fn test_selector() -> felt252 {
    selector!("TEST_CONTRACT_SELECTOR")
}

fn test_address() -> ContractAddress {
    contract_address_const::<469394814521890341860918960550914>()
}

/// Changes the block number for the given target and span.
/// - `target` - instance of `CheatTarget` specifying which contracts to roll
/// - `block_number` - block number to be set
/// - `span` - instance of `CheatSpan` specifying the number of target calls with the cheat applied
fn roll(target: CheatTarget, block_number: u64, span: CheatSpan) {
    validate_cheat_target_and_span(@target, @span);

    let mut inputs = array![];
    target.serialize(ref inputs);
    span.serialize(ref inputs);
    inputs.append(block_number.into());
    handle_cheatcode(cheatcode::<'roll'>(inputs.span()));
}

/// Changes the block number for the given target.
/// - `target` - instance of `CheatTarget` specifying which contracts to roll
/// - `block_number` - block number to be set
fn start_roll(target: CheatTarget, block_number: u64) {
    roll(target, block_number, CheatSpan::Indefinite);
}


/// Cancels the `roll` / `start_roll` for the given target.
/// - `target` - instance of `CheatTarget` specifying which contracts to stop rolling
fn stop_roll(target: CheatTarget) {
    let mut inputs = array![];
    target.serialize(ref inputs);
    handle_cheatcode(cheatcode::<'stop_roll'>(inputs.span()));
}

/// Changes the caller address for the given target and span.
/// - `target` - instance of `CheatTarget` specifying which contracts to prank
/// - `caller_address` - caller address to be set
/// - `span` - instance of `CheatSpan` specifying the number of target calls with the cheat applied
fn prank(target: CheatTarget, caller_address: ContractAddress, span: CheatSpan) {
    validate_cheat_target_and_span(@target, @span);

    let mut inputs = array![];
    target.serialize(ref inputs);
    span.serialize(ref inputs);
    inputs.append(caller_address.into());
    handle_cheatcode(cheatcode::<'prank'>(inputs.span()));
}

/// Changes the caller address for the given target.
/// This change can be canceled with `stop_prank`.
/// - `target` - instance of `CheatTarget` specifying which contracts to prank
/// - `caller_address` - caller address to be set
fn start_prank(target: CheatTarget, caller_address: ContractAddress) {
    prank(target, caller_address, CheatSpan::Indefinite);
}

/// Cancels the `prank` / `start_prank` for the given target.
/// - `target` - instance of `CheatTarget` specifying which contracts to stop pranking
fn stop_prank(target: CheatTarget) {
    let mut inputs = array![];
    target.serialize(ref inputs);
    handle_cheatcode(cheatcode::<'stop_prank'>(inputs.span()));
}

/// Changes the block timestamp for the given target and span.
/// - `target` - instance of `CheatTarget` specifying which contracts to warp
/// - `block_timestamp` - block timestamp to be set
/// - `span` - instance of `CheatSpan` specifying the number of target calls with the cheat applied
fn warp(target: CheatTarget, block_timestamp: u64, span: CheatSpan) {
    validate_cheat_target_and_span(@target, @span);

    let mut inputs = array![];
    target.serialize(ref inputs);
    span.serialize(ref inputs);
    inputs.append(block_timestamp.into());
    handle_cheatcode(cheatcode::<'warp'>(inputs.span()));
}

/// Changes the block timestamp for the given target.
/// - `target` - instance of `CheatTarget` specifying which contracts to warp
/// - `block_timestamp` - block timestamp to be set
fn start_warp(target: CheatTarget, block_timestamp: u64) {
    warp(target, block_timestamp, CheatSpan::Indefinite);
}

/// Cancels the `warp` / `start_warp` for the given target.
/// - `target` - instance of `CheatTarget` specifying which contracts to stop warping
fn stop_warp(target: CheatTarget) {
    let mut inputs = array![];
    target.serialize(ref inputs);
    handle_cheatcode(cheatcode::<'stop_warp'>(inputs.span()));
}

fn elect(target: CheatTarget, sequencer_address: ContractAddress, span: CheatSpan) {
    validate_cheat_target_and_span(@target, @span);

    let mut inputs = array![];
    target.serialize(ref inputs);
    span.serialize(ref inputs);
    inputs.append(sequencer_address.into());
    handle_cheatcode(cheatcode::<'elect'>(inputs.span()));
}

fn start_elect(target: CheatTarget, sequencer_address: ContractAddress) {
    elect(target, sequencer_address, CheatSpan::Indefinite);
}

fn stop_elect(target: CheatTarget) {
    let mut inputs = array![];
    target.serialize(ref inputs);
    handle_cheatcode(cheatcode::<'stop_elect'>(inputs.span()));
}


/// Mocks contract call to a `function_selector` of a contract at the given address, for `n_times` first calls that are made
/// to the contract.
/// A call to function `function_selector` will return data provided in `ret_data` argument.
/// An address with no contract can be mocked as well.
/// An entrypoint that is not present on the deployed contract is also possible to mock.
/// Note that the function is not meant for mocking internal calls - it works only for contract entry points.
/// - `contract_address` - target contract address
/// - `function_selector` - hashed name of the target function (can be obtained with `selector!` macro)
/// - `ret_data` - data to return by the function `function_selector`
/// - `n_times` - number of calls to mock the function for
fn mock_call<T, impl TSerde: core::serde::Serde<T>, impl TDestruct: Destruct<T>>(
    contract_address: ContractAddress, function_selector: felt252, ret_data: T, n_times: u32
) {
    assert!(n_times > 0, "cannot mock_call 0 times, n_times argument must be greater than 0");

    let contract_address_felt: felt252 = contract_address.into();
    let mut inputs = array![contract_address_felt, function_selector];

    CheatSpan::TargetCalls(n_times).serialize(ref inputs);

    let mut ret_data_arr = ArrayTrait::new();
    ret_data.serialize(ref ret_data_arr);

    ret_data_arr.serialize(ref inputs);

    handle_cheatcode(cheatcode::<'mock_call'>(inputs.span()));
}


/// Mocks contract call to a function of a contract at the given address, indefinitely.
/// See `mock_call` for comprehensive definition of how it can be used.
/// - `contract_address` - targeted contracts' address
/// - `function_selector` - hashed name of the target function (can be obtained with `selector!` macro)
/// - `ret_data` - data to be returned by the function
fn start_mock_call<T, impl TSerde: core::serde::Serde<T>, impl TDestruct: Destruct<T>>(
    contract_address: ContractAddress, function_selector: felt252, ret_data: T
) {
    let contract_address_felt: felt252 = contract_address.into();
    let mut inputs = array![contract_address_felt, function_selector];

    CheatSpan::Indefinite.serialize(ref inputs);

    let mut ret_data_arr = ArrayTrait::new();
    ret_data.serialize(ref ret_data_arr);

    ret_data_arr.serialize(ref inputs);

    handle_cheatcode(cheatcode::<'mock_call'>(inputs.span()));
}

/// Cancels the `mock_call` / `start_mock_call` for the function with given name and contract address.
/// - `contract_address` - targeted contracts' address
/// - `function_selector` - hashed name of the target function (can be obtained with `selector!` macro)
fn stop_mock_call(contract_address: ContractAddress, function_selector: felt252) {
    let contract_address_felt: felt252 = contract_address.into();
    handle_cheatcode(
        cheatcode::<'stop_mock_call'>(array![contract_address_felt, function_selector].span())
    );
}

fn replace_bytecode(contract: ContractAddress, new_class: ClassHash) {
    handle_cheatcode(
        cheatcode::<'replace_bytecode'>(array![contract.into(), new_class.into()].span())
    );
}

fn validate_cheat_target_and_span(target: @CheatTarget, span: @CheatSpan) {
    validate_cheat_span(span);

    if target == @CheatTarget::All {
        assert!(
            span == @CheatSpan::Indefinite,
            "CheatTarget::All can only be used with CheatSpan::Indefinite"
        );
    }
}

fn validate_cheat_span(span: @CheatSpan) {
    assert!(span != @CheatSpan::TargetCalls(0), "CheatSpan::TargetCalls must be greater than 0");
}
