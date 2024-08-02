use starknet::{ContractAddress, ClassHash, contract_address_const};
use starknet::testing::cheatcode;
use super::_cheatcode::handle_cheatcode;
use execution_info::{cheat_execution_info, ExecutionInfoMock, CheatArguments, Operation};

mod events;
mod l1_handler;
mod contract_class;
mod fork;
mod storage;
mod execution_info;
mod message_to_l1;

/// Enum used to specify how long the target should be cheated for.
#[derive(Copy, Drop, Serde, PartialEq, Clone, Debug)]
enum CheatSpan {
    /// Applies the cheatcode indefinitely, until the cheat is canceled manually (e.g. using
    /// `stop_cheat_block_timestamp`).
    Indefinite: (),
    /// Applies the cheatcode for a specified number of calls to the target,
    /// after which the cheat is canceled (or until the cheat is canceled manually).
    TargetCalls: usize,
}

fn test_selector() -> felt252 {
    // Result of selector!("TEST_CONTRACT_SELECTOR") since `selector!` macro requires dependency on `starknet`.
    655947323460646800722791151288222075903983590237721746322261907338444055163
}

fn test_address() -> ContractAddress {
    contract_address_const::<469394814521890341860918960550914>()
}

/// Mocks contract call to a `function_selector` of a contract at the given address, for `n_times`
/// first calls that are made to the contract.
/// A call to function `function_selector` will return data provided in `ret_data` argument.
/// An address with no contract can be mocked as well.
/// An entrypoint that is not present on the deployed contract is also possible to mock.
/// Note that the function is not meant for mocking internal calls - it works only for contract
/// entry points.
/// - `contract_address` - target contract address
/// - `function_selector` - hashed name of the target function (can be obtained with `selector!`
/// macro)
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
/// - `function_selector` - hashed name of the target function (can be obtained with `selector!`
/// macro)
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

/// Cancels the `mock_call` / `start_mock_call` for the function with given name and contract
/// address.
/// - `contract_address` - targeted contracts' address
/// - `function_selector` - hashed name of the target function (can be obtained with `selector!`
/// macro)
fn stop_mock_call(contract_address: ContractAddress, function_selector: felt252) {
    let contract_address_felt: felt252 = contract_address.into();
    handle_cheatcode(
        cheatcode::<'stop_mock_call'>(array![contract_address_felt, function_selector].span())
    );
}

#[derive(Drop, Serde, PartialEq, Debug)]
pub enum ReplaceBytecodeError {
    /// Means that the contract does not exist, and thus bytecode cannot be replaced
    ContractNotDeployed,
    /// Means that the given class for replacement is not declared
    UndeclaredClassHash,
}

/// Replaces class for given contract address.
/// The `new_class` hash has to be declared in order for the replacement class to execute the code,
/// when interacting with the contract.
/// - `contract` - address specifying which address will be replaced
/// - `new_class` - class hash, that will be used now for given address
/// Returns `Result::Ok` if the replacement succeeded, and a `ReplaceBytecodeError` with appropriate error type otherwise
fn replace_bytecode(
    contract: ContractAddress, new_class: ClassHash
) -> Result<(), ReplaceBytecodeError> {
    let mut cheat_result = handle_cheatcode(
        cheatcode::<'replace_bytecode'>(array![contract.into(), new_class.into()].span())
    );

    Serde::deserialize(ref cheat_result).unwrap()
}
