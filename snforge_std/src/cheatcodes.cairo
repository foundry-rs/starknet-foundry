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

#[derive(Copy, Drop, Serde, PartialEq, Clone, Debug, Display)]
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
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat_block_number
/// - `block_number` - block number to be set
/// - `span` - instance of `CheatSpan` specifying the number of target calls with the cheat applied
fn cheat_block_number(target: ContractAddress, block_number: u64, span: CheatSpan) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .block_info
        .block_number = Operation::Start(CheatArguments { value: block_number, span, target });

    cheat_execution_info(execution_info);
}

/// Changes the block number.
/// - `block_number` - block number to be set
fn cheat_block_number_global(block_number: u64) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.block_number = Operation::StartGlobal(block_number);

    cheat_execution_info(execution_info);
}

/// Cancels the `cheat_block_number_global`
fn stop_cheat_block_number_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.block_number = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the block number for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat_block_number
/// - `block_number` - block number to be set
fn start_cheat_block_number(target: ContractAddress, block_number: u64) {
    cheat_block_number(target, block_number, CheatSpan::Indefinite);
}


/// Cancels the `cheat_block_number` / `start_cheat_block_number` for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to stop cheat_block_numbering
fn stop_cheat_block_number(target: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.block_number = Operation::Stop(target);

    cheat_execution_info(execution_info);
}

/// Changes the caller address for the given target and span.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat_caller_address
/// - `caller_address` - caller address to be set
/// - `span` - instance of `CheatSpan` specifying the number of target calls with the cheat applied
fn cheat_caller_address(target: ContractAddress, caller_address: ContractAddress, span: CheatSpan) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .caller_address = Operation::Start(CheatArguments { value: caller_address, span, target });

    cheat_execution_info(execution_info);
}

/// Changes the caller address.
/// - `caller_address` - caller address to be set
fn cheat_caller_address_global(caller_address: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.caller_address = Operation::StartGlobal(caller_address);

    cheat_execution_info(execution_info);
}

/// Cancels the `cheat_caller_address_global`
fn stop_cheat_caller_address_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.caller_address = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the caller address for the given target.
/// This change can be canceled with `stop_cheat_caller_address`.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat_caller_address
/// - `caller_address` - caller address to be set
fn start_cheat_caller_address(target: ContractAddress, caller_address: ContractAddress) {
    cheat_caller_address(target, caller_address, CheatSpan::Indefinite);
}

/// Cancels the `cheat_caller_address` / `start_cheat_caller_address` for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to stop cheat_caller_addressing
fn stop_cheat_caller_address(target: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.caller_address = Operation::Stop(target);

    cheat_execution_info(execution_info);
}

/// Changes the block timestamp for the given target and span.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat_block_timestamp
/// - `block_timestamp` - block timestamp to be set
/// - `span` - instance of `CheatSpan` specifying the number of target calls with the cheat applied
fn cheat_block_timestamp(target: ContractAddress, block_timestamp: u64, span: CheatSpan) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .block_info
        .block_timestamp =
            Operation::Start(CheatArguments { value: block_timestamp, span, target });

    cheat_execution_info(execution_info);
}

/// Changes the block timestamp.
/// - `block_timestamp` - block timestamp to be set
fn cheat_block_timestamp_global(block_timestamp: u64) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.block_timestamp = Operation::StartGlobal(block_timestamp);

    cheat_execution_info(execution_info);
}

/// Cancels the `cheat_block_timestamp_global`
fn stop_cheat_block_timestamp_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.block_timestamp = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the block timestamp for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat_block_timestamp
/// - `block_timestamp` - block timestamp to be set
fn start_cheat_block_timestamp(target: ContractAddress, block_timestamp: u64) {
    cheat_block_timestamp(target, block_timestamp, CheatSpan::Indefinite);
}

/// Cancels the `cheat_block_timestamp` / `start_cheat_block_timestamp` for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to stop cheat_block_timestamping
fn stop_cheat_block_timestamp(target: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.block_timestamp = Operation::Stop(target);

    cheat_execution_info(execution_info);
}

/// Changes the sequencer address for the given target and span.
/// `target` - instance of `ContractAddress` specifying which contracts to cheat_sequencer_address
/// `sequencer_address` - sequencer address to be set
/// `span` - instance of `CheatSpan` specifying the number of target calls with the cheat applied
fn cheat_sequencer_address(
    target: ContractAddress, sequencer_address: ContractAddress, span: CheatSpan
) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info
        .block_info
        .sequencer_address =
            Operation::Start(CheatArguments { value: sequencer_address, span, target });

    cheat_execution_info(execution_info);
}

/// Changes the sequencer address.
/// `sequencer_address` - sequencer address to be set
fn cheat_sequencer_address_global(sequencer_address: ContractAddress) {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.sequencer_address = Operation::StartGlobal(sequencer_address);

    cheat_execution_info(execution_info);
}

/// Cancels the `cheat_sequencer_address_global`
fn stop_cheat_sequencer_address_global() {
    let mut execution_info: ExecutionInfoMock = Default::default();

    execution_info.block_info.sequencer_address = Operation::StopGlobal;

    cheat_execution_info(execution_info);
}

/// Changes the sequencer address for a given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to cheat_sequencer_address
/// - `sequencer_address` - sequencer address to be set
fn start_cheat_sequencer_address(target: ContractAddress, sequencer_address: ContractAddress) {
    cheat_sequencer_address(target, sequencer_address, CheatSpan::Indefinite);
}


/// Cancels the `cheat_sequencer_address` / `start_cheat_sequencer_address` for the given target.
/// - `target` - instance of `ContractAddress` specifying which contracts to stop cheat_sequencer_addressing
fn stop_cheat_sequencer_address(target: ContractAddress) {
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
