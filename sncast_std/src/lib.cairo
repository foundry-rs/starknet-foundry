use core::array::ArrayTrait;
use core::serde::Serde;
use starknet::{testing::cheatcode, ContractAddress, ClassHash};
use core::fmt::{Debug, Display, Error, Formatter};

#[derive(Drop, PartialEq, Serde, Debug)]
pub struct ErrorData {
    msg: ByteArray
}

#[derive(Drop, Serde, PartialEq, Debug)]
pub enum StarknetError {
    Other: ErrorData,
    ContractNotFound,
    ClassAlreadyDeclared,
    InsufficientMaxFee,
    InsufficientAccountBalance,
    ContractError: ErrorData,
    InvalidTransactionNonce,
    TransactionReverted: ErrorData,
    TransactionRejected
}

#[derive(Drop, Serde, PartialEq, Debug)]
pub enum ProviderError {
    Other: ErrorData,
    RateLimited,
    StarknetError: StarknetError,
}

#[derive(Drop, Serde, PartialEq, Debug)]
pub enum ScriptCommandError {
    ContractArtifactsNotFound: ErrorData,
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
    contract_address: ContractAddress, function_name: felt252, calldata: Array::<felt252>
) -> Result<CallResult, ScriptCommandError> {
    let contract_address_felt: felt252 = contract_address.into();
    let mut inputs = array![contract_address_felt, function_name];

    let mut calldata_serialized = array![];
    calldata.serialize(ref calldata_serialized);

    inputs.append_span(calldata_serialized.span());

    let mut buf = cheatcode::<'call'>(inputs.span());

    let mut result_data: Result<Array<felt252>, ScriptCommandError> = Serde::<
        Result<Array<felt252>>
    >::deserialize(ref buf)
        .expect('call deserialize failed');
    match result_data {
        Result::Ok(data) => Result::Ok(CallResult { data: data }),
        Result::Err(err) => Result::Err(err),
    }
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
    contract_name: felt252, max_fee: Option<felt252>, nonce: Option<felt252>
) -> Result<DeclareResult, ScriptCommandError> {
    let mut inputs = array![contract_name];

    let mut max_fee_serialized = array![];
    max_fee.serialize(ref max_fee_serialized);

    let mut nonce_serialized = array![];
    nonce.serialize(ref nonce_serialized);

    inputs.append_span(max_fee_serialized.span());
    inputs.append_span(nonce_serialized.span());

    let mut buf = cheatcode::<'declare'>(inputs.span());

    let mut result_data: Result<DeclareResult, ScriptCommandError> = Serde::<
        Result<DeclareResult>
    >::deserialize(ref buf)
        .expect('declare deserialize failed');

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

    let mut buf = cheatcode::<'deploy'>(inputs.span());

    let mut result_data: Result<DeployResult, ScriptCommandError> = Serde::<
        Result<DeployResult>
    >::deserialize(ref buf)
        .expect('deploy deserialize failed');

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

    let mut buf = cheatcode::<'invoke'>(inputs.span());

    let mut result_data: Result<InvokeResult, ScriptCommandError> = Serde::<
        Result<InvokeResult>
    >::deserialize(ref buf)
        .expect('invoke deserialize failed');

    result_data
}

pub fn get_nonce(block_tag: felt252) -> felt252 {
    let inputs = array![block_tag];
    let buf = cheatcode::<'get_nonce'>(inputs.span());
    *buf[0]
}
