use starknet::{ContractAddress, ClassHash, contract_address_const};
use super::_cheatcode::execute_cheatcode_and_deserialize;

pub mod events;
pub mod l1_handler;
pub mod contract_class;
pub mod storage;
pub mod execution_info;
pub mod message_to_l1;
pub mod generate_random_felt;

/// Enum used to specify how long the target should be cheated for.
#[derive(Copy, Drop, Serde, PartialEq, Clone, Debug)]
pub enum CheatSpan {
    /// Applies the cheatcode indefinitely, until the cheat is canceled manually (e.g. using
    /// `stop_cheat_block_timestamp`).
    Indefinite: (),
    /// Applies the cheatcode for a specified number of calls to the target,
    /// after which the cheat is canceled (or until the cheat is canceled manually).
    TargetCalls: usize,
}

/// Enum used to specify the call data that should be matched when mocking a contract call.
#[derive(Copy, Drop, PartialEq, Clone, Debug)]
pub enum MockCallData {
    /// Matches any call data.
    Any,
    /// Matches the specified serialized call data.
    Values: Span<felt252>,
}

impl MockCallDataSerde of Serde<MockCallData> {
    fn deserialize(ref serialized: Span<felt252>) -> Option<MockCallData> {
        let value: Option<Option<Span<felt252>>> = Serde::deserialize(ref serialized);

        match value {
            Option::None => Option::None,
            Option::Some(call_data) => match call_data {
                Option::None => Option::Some(MockCallData::Any),
                Option::Some(data) => Option::Some(MockCallData::Values(data)),
            },
        }
    }

    fn serialize(self: @MockCallData, ref output: Array<felt252>) {
        match self {
            MockCallData::Any => Option::<Span<felt252>>::None.serialize(ref output),
            MockCallData::Values(data) => Option::Some(*data).serialize(ref output),
        }
    }
}

pub fn test_selector() -> felt252 {
    // Result of selector!("TEST_CONTRACT_SELECTOR") since `selector!` macro requires dependency on
    // `starknet`.
    655947323460646800722791151288222075903983590237721746322261907338444055163
}

pub fn test_address() -> ContractAddress {
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
pub fn mock_call<T, impl TSerde: core::serde::Serde<T>, impl TDestruct: Destruct<T>>(
    contract_address: ContractAddress, function_selector: felt252, ret_data: T, n_times: u32
) {
    mock_call_when(contract_address, function_selector, MockCallData::Any, ret_data, n_times)
}

/// Mocks contract call to a function of a contract at the given address, indefinitely.
/// See `mock_call` for comprehensive definition of how it can be used.
/// - `contract_address` - targeted contracts' address
/// - `function_selector` - hashed name of the target function (can be obtained with `selector!`
/// macro)
/// - `ret_data` - data to be returned by the function
pub fn start_mock_call<T, impl TSerde: core::serde::Serde<T>, impl TDestruct: Destruct<T>>(
    contract_address: ContractAddress, function_selector: felt252, ret_data: T
) {
    start_mock_call_when(contract_address, function_selector, MockCallData::Any, ret_data)
}

/// Cancels the `mock_call` / `start_mock_call` for the function with given name and contract
/// address.
/// - `contract_address` - targeted contracts' address
/// - `function_selector` - hashed name of the target function (can be obtained with `selector!`
/// macro)
pub fn stop_mock_call(contract_address: ContractAddress, function_selector: felt252,) {
    stop_mock_call_when(contract_address, function_selector, MockCallData::Any)
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
/// - `call_data` - matching call data
/// - `ret_data` - data to return by the function `function_selector`
/// - `n_times` - number of calls to mock the function for
pub fn mock_call_when<T, impl TSerde: core::serde::Serde<T>, impl TDestruct: Destruct<T>>(
    contract_address: ContractAddress,
    function_selector: felt252,
    call_data: MockCallData,
    ret_data: T,
    n_times: u32
) {
    assert!(n_times > 0, "cannot mock_call 0 times, n_times argument must be greater than 0");

    let contract_address_felt: felt252 = contract_address.into();
    let mut inputs = array![contract_address_felt, function_selector];
    call_data.serialize(ref inputs);
    CheatSpan::TargetCalls(n_times).serialize(ref inputs);

    let mut ret_data_arr = ArrayTrait::new();
    ret_data.serialize(ref ret_data_arr);

    ret_data_arr.serialize(ref inputs);

    execute_cheatcode_and_deserialize::<'mock_call', ()>(inputs.span());
}


/// Mocks contract call to a function of a contract at the given address, indefinitely.
/// See `mock_call` for comprehensive definition of how it can be used.
/// - `contract_address` - targeted contracts' address
/// - `function_selector` - hashed name of the target function (can be obtained with `selector!`
/// macro)
/// - `call_data` - matching call data
/// - `ret_data` - data to be returned by the function
pub fn start_mock_call_when<T, impl TSerde: core::serde::Serde<T>, impl TDestruct: Destruct<T>>(
    contract_address: ContractAddress,
    function_selector: felt252,
    call_data: MockCallData,
    ret_data: T
) {
    let contract_address_felt: felt252 = contract_address.into();
    let mut inputs = array![contract_address_felt, function_selector];
    call_data.serialize(ref inputs);
    CheatSpan::Indefinite.serialize(ref inputs);

    let mut ret_data_arr = ArrayTrait::new();
    ret_data.serialize(ref ret_data_arr);

    ret_data_arr.serialize(ref inputs);

    execute_cheatcode_and_deserialize::<'mock_call', ()>(inputs.span());
}

/// Cancels the `mock_call_when` / `start_mock_call_when` for the function with given name and
/// contract address.
/// - `contract_address` - targeted contracts' address
/// - `function_selector` - hashed name of the target function (can be obtained with `selector!`
/// - `call_data` - matching call data
/// macro)
pub fn stop_mock_call_when(
    contract_address: ContractAddress, function_selector: felt252, call_data: MockCallData
) {
    let contract_address_felt: felt252 = contract_address.into();
    let mut inputs = array![contract_address_felt, function_selector];
    call_data.serialize(ref inputs);

    execute_cheatcode_and_deserialize::<'stop_mock_call', ()>(inputs.span());
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
/// Returns `Result::Ok` if the replacement succeeded, and a `ReplaceBytecodeError` with appropriate
/// error type otherwise
pub fn replace_bytecode(
    contract: ContractAddress, new_class: ClassHash
) -> Result<(), ReplaceBytecodeError> {
    execute_cheatcode_and_deserialize::<
        'replace_bytecode'
    >(array![contract.into(), new_class.into()].span())
}
