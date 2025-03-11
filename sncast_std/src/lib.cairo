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
    pub transaction_index: felt252,
    pub execution_error: ByteArray,
}

#[derive(Drop, Serde, PartialEq, Debug)]
pub enum StarknetError {
    /// Failed to receive transaction
    FailedToReceiveTransaction,
    /// Contract not found
    ContractNotFound,
    /// Requested entrypoint does not exist in the contract
    EntryPointNotFound,
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
    /// The transaction's resources don't cover validation or the minimal transaction fee
    InsufficientResourcesForValidate,
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

#[derive(Drop, Copy, Debug, Serde)]
pub enum DeclareResult {
    AlreadyDeclared: AlreadyDeclaredResult,
    Success: DeclareTransactionResult,
}

#[derive(Drop, Copy, Debug, Serde)]
pub struct DeclareTransactionResult {
    pub class_hash: ClassHash,
    pub transaction_hash: felt252,
}

#[derive(Drop, Copy, Debug, Serde)]
pub struct AlreadyDeclaredResult {
    pub class_hash: ClassHash,
}

pub trait DeclareResultTrait {
    fn class_hash(self: @DeclareResult) -> @ClassHash;
}

impl DeclareResultImpl of DeclareResultTrait {
    fn class_hash(self: @DeclareResult) -> @ClassHash {
        match self {
            DeclareResult::Success(result) => result.class_hash,
            DeclareResult::AlreadyDeclared(result) => result.class_hash
        }
    }
}

impl DisplayDeclareResult of Display<DeclareResult> {
    fn fmt(self: @DeclareResult, ref f: Formatter) -> Result<(), Error> {
        match self {
            DeclareResult::Success(result) => write!(
                f,
                "class_hash: {}, transaction_hash: {}",
                result.class_hash,
                result.transaction_hash
            ),
            DeclareResult::AlreadyDeclared(result) => write!(f, "class_hash: {}", result.class_hash)
        }
    }
}

pub fn declare(
    contract_name: ByteArray, fee_settings: FeeSettings, nonce: Option<felt252>
) -> Result<DeclareResult, ScriptCommandError> {
    let mut inputs = array![];

    contract_name.serialize(ref inputs);

    let mut fee_settings_serialized = array![];
    fee_settings.serialize(ref fee_settings_serialized);

    let mut nonce_serialized = array![];
    nonce.serialize(ref nonce_serialized);

    inputs.append_span(fee_settings_serialized.span());
    inputs.append_span(nonce_serialized.span());

    let mut buf = handle_cheatcode(cheatcode::<'declare'>(inputs.span()));

    let mut result_data: Result<DeclareResult, ScriptCommandError> =
        match Serde::<Result<DeclareResult>>::deserialize(ref buf) {
        Option::Some(result_data) => result_data,
        Option::None => panic!("declare deserialize failed"),
    };

    result_data
}

#[derive(Drop, Copy, Debug, Serde)]
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
#[derive(Drop, Copy, Debug, Serde, PartialEq)]
pub struct FeeSettings {
    pub max_fee: Option<felt252>,
    pub l1_gas: Option<u64>,
    pub l1_gas_price: Option<u128>,
    pub l2_gas: Option<u64>,
    pub l2_gas_price: Option<u128>,
    pub l1_data_gas: Option<u64>,
    pub l2_data_gas_price: Option<u128>,
}

pub fn deploy(
    class_hash: ClassHash,
    constructor_calldata: Array::<felt252>,
    salt: Option<felt252>,
    unique: bool,
    fee_settings: FeeSettings,
    nonce: Option<felt252>
) -> Result<DeployResult, ScriptCommandError> {
    let class_hash_felt: felt252 = class_hash.into();
    let mut inputs = array![class_hash_felt];

    let mut constructor_calldata_serialized = array![];
    constructor_calldata.serialize(ref constructor_calldata_serialized);

    let mut salt_serialized = array![];
    salt.serialize(ref salt_serialized);

    let mut fee_settings_serialized = array![];
    fee_settings.serialize(ref fee_settings_serialized);

    let mut nonce_serialized = array![];
    nonce.serialize(ref nonce_serialized);

    inputs.append_span(constructor_calldata_serialized.span());
    inputs.append_span(salt_serialized.span());
    inputs.append(unique.into());
    inputs.append_span(fee_settings_serialized.span());
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
    fee_settings: FeeSettings,
    nonce: Option<felt252>
) -> Result<InvokeResult, ScriptCommandError> {
    let contract_address_felt: felt252 = contract_address.into();
    let mut inputs = array![contract_address_felt, entry_point_selector];

    let mut calldata_serialized = array![];
    calldata.serialize(ref calldata_serialized);

    let mut fee_settings_serialized = array![];
    fee_settings.serialize(ref fee_settings_serialized);

    let mut nonce_serialized = array![];
    nonce.serialize(ref nonce_serialized);

    inputs.append_span(calldata_serialized.span());
    inputs.append_span(fee_settings_serialized.span());
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

#[derive(Drop, Copy, Debug, Serde, PartialEq)]
pub enum FinalityStatus {
    Received,
    Rejected,
    AcceptedOnL2,
    AcceptedOnL1
}

pub impl DisplayFinalityStatus of Display<FinalityStatus> {
    fn fmt(self: @FinalityStatus, ref f: Formatter) -> Result<(), Error> {
        let finality_status: ByteArray = match self {
            FinalityStatus::Received => "Received",
            FinalityStatus::Rejected => "Rejected",
            FinalityStatus::AcceptedOnL2 => "AcceptedOnL2",
            FinalityStatus::AcceptedOnL1 => "AcceptedOnL1",
        };
        write!(f, "{finality_status}")
    }
}


#[derive(Drop, Copy, Debug, Serde, PartialEq)]
pub enum ExecutionStatus {
    Succeeded,
    Reverted,
}

pub impl DisplayExecutionStatus of Display<ExecutionStatus> {
    fn fmt(self: @ExecutionStatus, ref f: Formatter) -> Result<(), Error> {
        let execution_status: ByteArray = match self {
            ExecutionStatus::Succeeded => "Succeeded",
            ExecutionStatus::Reverted => "Reverted"
        };
        write!(f, "{execution_status}")
    }
}


#[derive(Drop, Copy, Debug, Serde, PartialEq)]
pub struct TxStatusResult {
    pub finality_status: FinalityStatus,
    pub execution_status: Option<ExecutionStatus>
}

pub impl DisplayTxStatusResult of Display<TxStatusResult> {
    fn fmt(self: @TxStatusResult, ref f: Formatter) -> Result<(), Error> {
        match self.execution_status {
            Option::Some(status) => write!(
                f, "finality_status: {}, execution_status: {}", self.finality_status, status
            ),
            Option::None => write!(f, "finality_status: {}", self.finality_status),
        }
    }
}

pub fn tx_status(transaction_hash: felt252) -> Result<TxStatusResult, ScriptCommandError> {
    let mut inputs = array![transaction_hash];

    let mut buf = handle_cheatcode(cheatcode::<'tx_status'>(inputs.span()));

    let mut result_data: Result<TxStatusResult, ScriptCommandError> =
        match Serde::<Result<TxStatusResult>>::deserialize(ref buf) {
        Option::Some(result_data) => result_data,
        Option::None => panic!("tx_status deserialize failed")
    };

    result_data
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
