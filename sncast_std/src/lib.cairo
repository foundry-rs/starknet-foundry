use core::array::ArrayTrait;
use core::serde::Serde;
use starknet::{testing::cheatcode, ContractAddress, ClassHash};
use core::fmt::{Debug, Display, Error, Formatter};

#[derive(Drop, PartialEq, Serde, Debug)]
pub struct ErrorData {
    msg: ByteArray
}

#[derive(Drop, PartialEq, Serde, Debug)]
pub struct TransactionExecutionErrorData {
    transaction_index: felt252,
    execution_error: ByteArray,
}

#[derive(Drop, Serde, PartialEq, Debug)]
pub enum StarknetError {
    /// Failed to receive transaction
    FailedToReceiveTransaction,
    /// Contract not found
    ContractNotFound,
    /// Block not found
    BlockNotFound,
    /// Invalid transaction index in a block
    InvalidTransactionIndex,
    /// Class hash not found
    ClassHashNotFound,
    /// Transaction hash not found
    TransactionHashNotFound,
    /// Contract error
    ContractError: ErrorData,
    /// Transaction execution error
    TransactionExecutionError: TransactionExecutionErrorData,
    /// Class already declared
    ClassAlreadyDeclared,
    /// Invalid transaction nonce
    InvalidTransactionNonce,
    /// Max fee is smaller than the minimal transaction cost (validation plus fee transfer)
    InsufficientMaxFee,
    /// Account balance is smaller than the transaction's max_fee
    InsufficientAccountBalance,
    /// Account validation failed
    ValidationFailure: ErrorData,
    /// Compilation failed
    CompilationFailed,
    /// Contract class size it too large
    ContractClassSizeIsTooLarge,
    /// Sender address in not an account contract
    NonAccount,
    /// A transaction with the same hash already exists in the mempool
    DuplicateTx,
    /// the compiled class hash did not match the one supplied in the transaction
    CompiledClassHashMismatch,
    /// the transaction version is not supported
    UnsupportedTxVersion,
    /// the contract class version is not supported
    UnsupportedContractClassVersion,
    /// An unexpected error occurred
    UnexpectedError: ErrorData,
}

#[derive(Drop, Serde, PartialEq, Debug)]
pub enum ProviderError {
    StarknetError: StarknetError,
    RateLimited,
    UnknownError: ErrorData,
}

#[derive(Drop, Serde, PartialEq, Debug)]
pub enum TransactionError {
    Rejected,
    Reverted: ErrorData,
}

#[derive(Drop, Serde, PartialEq, Debug)]
pub enum WaitForTransactionError {
    TransactionError: TransactionError,
    TimedOut,
    ProviderError: ProviderError,
}

#[derive(Drop, Serde, PartialEq, Debug)]
pub enum ScriptCommandError {
    UnknownError: ErrorData,
    ContractArtifactsNotFound: ErrorData,
    WaitForTransactionError: WaitForTransactionError,
    ProviderError: ProviderError,
}

pub impl DisplayClassHash of Display<ClassHash> {
    fn fmt(self: @ClassHash, ref f: Formatter) -> Result<(), Error> {
        let class_hash: felt252 = (*self).into();
        Display::fmt(@class_hash, ref f)
    }
}

pub impl DisplayContractAddress of Display<ContractAddress> {
    fn fmt(self: @ContractAddress, ref f: Formatter) -> Result<(), Error> {
        let addr: felt252 = (*self).into();
        Display::fmt(@addr, ref f)
    }
}

#[derive(Drop, Clone, Debug, Serde)]
pub struct CallResult {
    pub data: Array::<felt252>,
}

impl DisplayCallResult of Display<CallResult> {
    fn fmt(self: @CallResult, ref f: Formatter) -> Result<(), Error> {
        Debug::fmt(self.data, ref f)
    }
}

pub fn call(
    contract_address: ContractAddress, function_selector: felt252, calldata: Array::<felt252>
) -> Result<CallResult, ScriptCommandError> {
    let contract_address_felt: felt252 = contract_address.into();
    let mut inputs = array![contract_address_felt, function_selector];

    let mut calldata_serialized = array![];
    calldata.serialize(ref calldata_serialized);

    inputs.append_span(calldata_serialized.span());

    let mut buf = handle_cheatcode(cheatcode::<'call'>(inputs.span()));

    let mut result_data: Result<CallResult, ScriptCommandError> =
        match Serde::<Result<CallResult>>::deserialize(ref buf) {
        Option::Some(result_data) => result_data,
        Option::None => panic!("call deserialize failed")
    };

    result_data
}

#[derive(Drop, Clone, Debug, Serde)]
pub struct DeclareResult {
    pub class_hash: ClassHash,
    pub transaction_hash: felt252,
}

impl DisplayDeclareResult of Display<DeclareResult> {
    fn fmt(self: @DeclareResult, ref f: Formatter) -> Result<(), Error> {
        write!(f, "class_hash: {}, transaction_hash: {}", *self.class_hash, *self.transaction_hash)
    }
}

pub fn declare(
    contract_name: ByteArray, max_fee: Option<felt252>, nonce: Option<felt252>
) -> Result<DeclareResult, ScriptCommandError> {
    // it's in fact core::byte_array::BYTE_ARRAY_MAGIC but it can't be imported here
    let mut inputs = array![0x46a6158a16a947e5916b2a2ca68501a45e93d7110e81aa2d6438b1c57c879a3];

    contract_name.serialize(ref inputs);

    let mut max_fee_serialized = array![];
    max_fee.serialize(ref max_fee_serialized);

    let mut nonce_serialized = array![];
    nonce.serialize(ref nonce_serialized);

    inputs.append_span(max_fee_serialized.span());
    inputs.append_span(nonce_serialized.span());

    let mut buf = handle_cheatcode(cheatcode::<'declare'>(inputs.span()));

    let mut result_data: Result<DeclareResult, ScriptCommandError> =
        match Serde::<Result<DeclareResult>>::deserialize(ref buf) {
        Option::Some(result_data) => result_data,
        Option::None => panic!("declare deserialize failed"),
    };

    result_data
}

#[derive(Drop, Clone, Debug, Serde)]
pub struct DeployResult {
    pub contract_address: ContractAddress,
    pub transaction_hash: felt252,
}

impl DisplayDeployResult of Display<DeployResult> {
    fn fmt(self: @DeployResult, ref f: Formatter) -> Result<(), Error> {
        write!(
            f,
            "contract_address: {}, transaction_hash: {}",
            *self.contract_address,
            *self.transaction_hash
        )
    }
}

pub fn deploy(
    class_hash: ClassHash,
    constructor_calldata: Array::<felt252>,
    salt: Option<felt252>,
    unique: bool,
    max_fee: Option<felt252>,
    nonce: Option<felt252>
) -> Result<DeployResult, ScriptCommandError> {
    let class_hash_felt: felt252 = class_hash.into();
    let mut inputs = array![class_hash_felt];

    let mut constructor_calldata_serialized = array![];
    constructor_calldata.serialize(ref constructor_calldata_serialized);

    let mut salt_serialized = array![];
    salt.serialize(ref salt_serialized);

    let mut max_fee_serialized = array![];
    max_fee.serialize(ref max_fee_serialized);

    let mut nonce_serialized = array![];
    nonce.serialize(ref nonce_serialized);

    inputs.append_span(constructor_calldata_serialized.span());
    inputs.append_span(salt_serialized.span());
    inputs.append(unique.into());
    inputs.append_span(max_fee_serialized.span());
    inputs.append_span(nonce_serialized.span());

    let mut buf = handle_cheatcode(cheatcode::<'deploy'>(inputs.span()));

    let mut result_data: Result<DeployResult, ScriptCommandError> =
        match Serde::<Result<DeployResult>>::deserialize(ref buf) {
        Option::Some(result_data) => result_data,
        Option::None => panic!("deploy deserialize failed")
    };

    result_data
}

#[derive(Drop, Clone, Debug, Serde)]
pub struct InvokeResult {
    pub transaction_hash: felt252,
}

impl DisplayInvokeResult of Display<InvokeResult> {
    fn fmt(self: @InvokeResult, ref f: Formatter) -> Result<(), Error> {
        write!(f, "{}", *self.transaction_hash)
    }
}

pub fn invoke(
    contract_address: ContractAddress,
    entry_point_selector: felt252,
    calldata: Array::<felt252>,
    max_fee: Option<felt252>,
    nonce: Option<felt252>
) -> Result<InvokeResult, ScriptCommandError> {
    let contract_address_felt: felt252 = contract_address.into();
    let mut inputs = array![contract_address_felt, entry_point_selector];

    let mut calldata_serialized = array![];
    calldata.serialize(ref calldata_serialized);

    let mut max_fee_serialized = array![];
    max_fee.serialize(ref max_fee_serialized);

    let mut nonce_serialized = array![];
    nonce.serialize(ref nonce_serialized);

    inputs.append_span(calldata_serialized.span());
    inputs.append_span(max_fee_serialized.span());
    inputs.append_span(nonce_serialized.span());

    let mut buf = handle_cheatcode(cheatcode::<'invoke'>(inputs.span()));

    let mut result_data: Result<InvokeResult, ScriptCommandError> =
        match Serde::<Result<InvokeResult>>::deserialize(ref buf) {
        Option::Some(result_data) => result_data,
        Option::None => panic!("invoke deserialize failed")
    };

    result_data
}

pub fn get_nonce(block_tag: felt252) -> felt252 {
    let inputs = array![block_tag];
    let buf = handle_cheatcode(cheatcode::<'get_nonce'>(inputs.span()));
    *buf[0]
}

fn handle_cheatcode(input: Span<felt252>) -> Span<felt252> {
    let first = *input.at(0);
    let input = input.slice(1, input.len() - 1);

    if first == 1 {
        // it's in fact core::byte_array::BYTE_ARRAY_MAGIC but it can't be imported here
        let mut arr = array![0x46a6158a16a947e5916b2a2ca68501a45e93d7110e81aa2d6438b1c57c879a3];

        arr.append_span(input);

        panic(arr)
    } else {
        input
    }
}
