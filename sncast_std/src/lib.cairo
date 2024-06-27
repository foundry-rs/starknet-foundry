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

/// Calls a contract
/// - `contract_address` - address of the contract which will be called
/// - `function_selector` - hashed name of the target function (can be obtained with `selector!` macro)
/// - `calldata` - calldata for the function inputs, serialized with `Serde`
/// Returns [`CallResult`] with the returned data from the called function, or  [`ScriptCommandError`] in case of failure
///
/// Usage example:
/// ```
///  use sncast_std::{call, CallResult};
///  use starknet::{ContractAddress};
///
///  fn main() {
///      let contract_address: ContractAddress = 0x1e52f6ebc3e594d2a6dc2a0d7d193cb50144cfdfb7fdd9519135c29b67e427
///          .try_into()
///          .expect('Invalid contract address value');
///
///      let call_result = call(contract_address, selector!("get"), array![0x1]).expect('call failed');
///      println!("call_result: {}", call_result);
///      println!("debug call_result: {:?}", call_result);
///  }
/// ```
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

/// Result of successfully submitting a `Declare` transaction
#[derive(Drop, Clone, Debug, Serde)]
pub struct DeclareResult {
    /// Resulting class hash which was declared during the transaction
    pub class_hash: ClassHash,
    /// A hash which references the transaction in which the declaration was made
    pub transaction_hash: felt252,
}

impl DisplayDeclareResult of Display<DeclareResult> {
    fn fmt(self: @DeclareResult, ref f: Formatter) -> Result<(), Error> {
        write!(f, "class_hash: {}, transaction_hash: {}", *self.class_hash, *self.transaction_hash)
    }
}
/// Declares a contract from the project on the network
/// - `contract_name` - name of a contract as Cairo string.
///     It is a name of the contract (part after `mod` keyword) e.g. `"HelloStarknet"`.
/// - `max_fee` - The fee in tokens you're willing to pay for the transaction.
///     If not provided, max fee will be automatically estimated via estimation endpoint.
/// - `nonce` - Account nonce for declare transaction.
///     If not provided, nonce will be set automatically (by fetching it from the account contract in the background).
/// Returns [`DeclareResult`] if declare was successful or a [`ScriptCommandError`] in case of failure
///
/// Usage example:
/// ```
/// use sncast_std::{declare, DeclareResult};
///
/// fn main() {
///     let max_fee = 9999999;
///     let declare_result = declare("HelloStarknet", Option::Some(max_fee), Option::None).expect('declare failed');
///
///     println!("declare_result: {}", declare_result);
///     println!("debug declare_result: {:?}", declare_result);
/// }
/// ```
pub fn declare(
    contract_name: ByteArray, max_fee: Option<felt252>, nonce: Option<felt252>
) -> Result<DeclareResult, ScriptCommandError> {
    let mut inputs = array![];

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

/// Submits a deployment transaction, via invocation of Universal Deployer Contract (UDC)
/// - `class_hash` - class hash (retrieved from [`declare`]) of a contract to deploy
/// - `constructor_calldata` - calldata for the constructor, serialized with `Serde`
/// - `salt` - optional salt for the contract address, used for providing address uniqueness
/// - `unique` - determines if the deployment should be origin dependent or not.
///     Origin independent calculation only takes into account class hash, salt and constructor arguments.
///     Origin dependent address calculation additionally includes account address and UDC address
///     in the resulting contracts' address.
/// - `max_fee` - The fee in tokens you're willing to pay for the transaction.
///     If not provided, max fee will be automatically estimated via estimation endpoint.
/// - `nonce` - Account nonce for declare transaction.
///     If not provided, nonce will be set automatically (by fetching it from the account contract in the background).
/// Returns `DeployResult` or `ScriptCommandError` in case of failure
///
/// Usage example:
/// ```
/// use sncast_std::{deploy, DeployResult};
///
/// fn main() {
///     let max_fee = 9999999;
///     let salt = 0x1;
///     let nonce = 0x1;
///     let class_hash: ClassHash = 0x03a8b191831033ba48ee176d5dde7088e71c853002b02a1cfa5a760aa98be046
///         .try_into()
///         .expect('Invalid class hash value');
///
///     let deploy_result = deploy(
///         class_hash,
///         ArrayTrait::new(),
///         Option::Some(salt),
///         true,
///         Option::Some(max_fee),
///         Option::Some(nonce)
///     ).expect('deploy failed');
///
///     println!("deploy_result: {}", deploy_result);
///     println!("debug deploy_result: {:?}", deploy_result);
/// }
/// ```
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

/// Submits a transaction with given contracts' function invocation
/// - `contract_address` - address of the contract which will be invoked
/// - `entry_point_selector` - hashed name of the target function (can be obtained with `selector!` macro)
/// - `calldata` - inputs for the invoked function, serialized with `Serde`
/// - `max_fee` - The fee in tokens you're willing to pay for the transaction.
///     If not provided, max fee will be automatically estimated via estimation endpoint.
/// - `nonce` - Account nonce for declare transaction.
///     If not provided, nonce will be set automatically (by fetching it from the account contract in the background).
/// Returns `InvokeResult` or `ScriptCommandError` if *submitting* the transaction failed
///
/// Usage example:
/// ```
/// use sncast_std::{invoke, InvokeResult};
/// use starknet::{ContractAddress};
///
/// fn main() {
///     let contract_address: ContractAddress = 0x1e52f6ebc3e594d2a6dc2a0d7d193cb50144cfdfb7fdd9519135c29b67e427
///         .try_into()
///         .expect('Invalid contract address value');
///
///     let invoke_result = invoke(
///         contract_address, selector!("put"), array![0x1, 0x2], Option::None, Option::None
///     ).expect('invoke failed');
///
///     println!("invoke_result: {}", invoke_result);
///     println!("debug invoke_result: {:?}", invoke_result);
/// }
/// ```
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

/// Gets nonce of an account for a given block tag
/// - `block_tag` - block tag name in form of shortstring (one of 'pending' or 'latest').
/// Returns active accounts' nonce as `felt252`.
///
/// Usage example:
/// ```
/// use sncast_std::{get_nonce};
///
/// fn main() {
///     let nonce = get_nonce('latest');
///     println!("nonce: {}", nonce);
///     println!("debug nonce: {:?}", nonce);
/// }
/// ```
pub fn get_nonce(block_tag: felt252) -> felt252 {
    let inputs = array![block_tag];
    let buf = handle_cheatcode(cheatcode::<'get_nonce'>(inputs.span()));
    *buf[0]
}

/// Represents the status in the transaction lifecycle and its' User -> L2 -> L1 journey
#[derive(Drop, Clone, Debug, Serde, PartialEq)]
pub enum FinalityStatus {
    /// Transaction was received by the node successfully
    Received,
    /// Transaction was rejected on stage of validation by sequencer
    Rejected,
    /// Transaction was executed and entered an actual created block on L2
    AcceptedOnL2,
    /// The transaction was accepted on Ethereum
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


/// Result of cairo code execution for the transaction after validation
#[derive(Drop, Copy, Debug, Serde, PartialEq)]
pub enum ExecutionStatus {
    /// The transaction was successfully executed by the sequencer
    Succeeded,
    /// The transaction passed validation but failed during execution in the sequencer
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

/// A structure representing status of the transaction execution after its' submission
#[derive(Drop, Clone, Debug, Serde, PartialEq)]
pub struct TxStatusResult {
    /// Represents the status of the transaction in the L2 -> L1 pipeline
    pub finality_status: FinalityStatus,
    /// Result of the cairo code execution. Present when the transaction was actually included in the block.
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


/// Gets the status of a transaction. Useful for assessment of completion status for the transaction.
/// - `transaction_hash` - A `felt252` representing hash of the transaction (i.e. result of `deploy`/`invoke`/`declare`)
///  Returns `TxStatusResult` or `ScriptCommandError` if retrieving the status failed
///
/// Usage example:
/// ```
/// use sncast_std::{tx_status};
///
/// fn main() {
///     let transaction_hash = 0x00ae35dacba17cde62b8ceb12e3b18f4ab6e103fa2d5e3d9821cb9dc59d59a3c;
///     let status = tx_status(transaction_hash).expect("Failed to get transaction status");
///
///     println!("transaction status: {:?}", status);
/// }
/// ```
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
