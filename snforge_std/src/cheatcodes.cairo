use starknet::{ContractAddress, ClassHash, contract_address_const};
use starknet::testing::cheatcode;
use super::_cheatcode::handle_cheatcode;
use execution_info::{cheat_execution_info, ExecutionInfoMock, CheatArguments, Operation};

mod events;
mod l1_handler;
mod contract_class;
mod tx_info;
mod fork;
mod storage;
mod execution_info;

/// Enum used to specify how long the target should be cheated for.
#[derive(Copy, Drop, Serde, PartialEq, Clone, Debug, Display)]
enum CheatSpan {
    /// Applies the cheatcode indefinitely, until the cheat is canceled manually (e.g. using `stop_warp`).
    Indefinite: (),
    /// Applies the cheatcode for a specified number of calls to the target,
    /// after which the cheat is canceled (or until the cheat is canceled manually).
    TargetCalls: usize,
}

fn test_selector() -> felt252 {
    selector!("TEST_CONTRACT_SELECTOR")
}

fn test_address() -> ContractAddress {
    contract_address_const::<469394814521890341860918960550914>()
}

/// Changes the block number for the given target and span.
/// - `target` - instance of `ContractAddress` specifying which contracts to roll
/// - `block_number` - block number to be set
/// - `span` - instance of `CheatSpan` specifying the number of target calls with the cheat applied
fn roll(target: ContractAddress, block_number: u64, span: CheatSpan) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .block_info
        .block_number = Operation::Start(CheatArguments { value: block_number, span, target });

    cheat_execution_info(execution_info);
}

/// Changes the block number.
/// - `block_number` - block number to be set
fn roll_global(block_number: u64) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.block_number = Operation::StartGlobal(block_number);

    cheat_execution_info(execution_info);
}

/// Cancels the `roll_global`
fn stop_roll_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.block_number = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the block number for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to roll
/// - `block_number` - block number to be set
fn start_roll(target: ContractAddress, block_number: u64) {
    roll(target, block_number, CheatSpan::Indefinite);
}


/// Cancels the `roll` / `start_roll` for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to stop rolling
fn stop_roll(target: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.block_number = Operation::Stop(target);

    cheat_execution_info(execution_info);
}

/// Changes the caller address for the given target and span.
/// - `target` - instance of `ContractAddress` specifying which contracts to prank
/// - `caller_address` - caller address to be set
/// - `span` - instance of `CheatSpan` specifying the number of target calls with the cheat applied
fn prank(target: ContractAddress, caller_address: ContractAddress, span: CheatSpan) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .caller_address = Operation::Start(CheatArguments { value: caller_address, span, target });

    cheat_execution_info(execution_info);
}

/// Changes the caller address.
/// - `caller_address` - caller address to be set
fn prank_global(caller_address: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.caller_address = Operation::StartGlobal(caller_address);

    cheat_execution_info(execution_info);
}

/// Cancels the `prank_global`
fn stop_prank_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.caller_address = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the caller address for the given target.
/// This change can be canceled with `stop_prank`.
/// - `target` - instance of `ContractAddress` specifying which contracts to prank
/// - `caller_address` - caller address to be set
fn start_prank(target: ContractAddress, caller_address: ContractAddress) {
    prank(target, caller_address, CheatSpan::Indefinite);
}

/// Cancels the `prank` / `start_prank` for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to stop pranking
fn stop_prank(target: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.caller_address = Operation::Stop(target);

    cheat_execution_info(execution_info);
}

/// Changes the block timestamp for the given target and span.
/// - `target` - instance of `ContractAddress` specifying which contracts to warp
/// - `block_timestamp` - block timestamp to be set
/// - `span` - instance of `CheatSpan` specifying the number of target calls with the cheat applied
fn warp(target: ContractAddress, block_timestamp: u64, span: CheatSpan) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .block_info
        .block_timestamp =
            Operation::Start(CheatArguments { value: block_timestamp, span, target });

    cheat_execution_info(execution_info);
}

/// Changes the block timestamp.
/// - `block_timestamp` - block timestamp to be set
fn warp_global(block_timestamp: u64) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.block_timestamp = Operation::StartGlobal(block_timestamp);

    cheat_execution_info(execution_info);
}

/// Cancels the `warp_global`
fn stop_warp_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.block_timestamp = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the block timestamp for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to warp
/// - `block_timestamp` - block timestamp to be set
fn start_warp(target: ContractAddress, block_timestamp: u64) {
    warp(target, block_timestamp, CheatSpan::Indefinite);
}

/// Cancels the `warp` / `start_warp` for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to stop warping
fn stop_warp(target: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.block_timestamp = Operation::Stop(target);

    cheat_execution_info(execution_info);
}

/// Changes the sequencer address for the given target and span.
/// `target` - instance of `ContractAddress` specifying which contracts to elect
/// `sequencer_address` - sequencer address to be set
/// `span` - instance of `CheatSpan` specifying the number of target calls with the cheat applied
fn elect(target: ContractAddress, sequencer_address: ContractAddress, span: CheatSpan) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .block_info
        .sequencer_address =
            Operation::Start(CheatArguments { value: sequencer_address, span, target });

    cheat_execution_info(execution_info);
}

/// Changes the sequencer address.
/// `sequencer_address` - sequencer address to be set
fn elect_global(sequencer_address: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.sequencer_address = Operation::StartGlobal(sequencer_address);

    cheat_execution_info(execution_info);
}

/// Cancels the `elect_global`
fn stop_elect_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.sequencer_address = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the sequencer address for a given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to elect
/// - `sequencer_address` - sequencer address to be set
fn start_elect(target: ContractAddress, sequencer_address: ContractAddress) {
    elect(target, sequencer_address, CheatSpan::Indefinite);
}


/// Cancels the `elect` / `start_elect` for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to stop electing
fn stop_elect(target: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.sequencer_address = Operation::Stop(target);

    cheat_execution_info(execution_info);
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

/// Replaces class for given contract address.
/// The `new_class` hash has to be declared in order for the replacement class to execute the code,
/// when interacting with the contract.
/// - `contract` - address specifying which address will be replaced
/// - `new_class` - class hash, that will be used now for given address
fn replace_bytecode(contract: ContractAddress, new_class: ClassHash) {
    handle_cheatcode(
        cheatcode::<'replace_bytecode'>(array![contract.into(), new_class.into()].span())
    );
}
