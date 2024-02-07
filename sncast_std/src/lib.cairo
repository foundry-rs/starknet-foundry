use core::array::ArrayTrait;
use core::serde::Serde;
use core::debug::PrintTrait;
use starknet::{testing::cheatcode, ContractAddress, ClassHash};


#[derive(Copy, Drop, Serde, PartialEq)]
enum StarknetError {
    UnknownError,
    ContractNotFound,
    BlockNotFound,
    ClassHashNotFound,
    ClassAlreadyDeclared,
    InsufficientMaxFee,
    InsufficientAccountBalance,
    ContractError,
    InvalidTransactionNonce,
    ContractAddressUnavailableForDeployment,
    ClassNotDeclared,
    TransactionReverted,
}

impl StarknetErrorTrait of PrintTrait<StarknetError> {
    #[inline(always)]
    fn print(self: StarknetError) {
        match self {
            StarknetError::UnknownError => { 'StarknetUnknownError'.print(); },
            StarknetError::ContractNotFound => { 'ContractNotFound'.print(); },
            StarknetError::BlockNotFound => { 'BlockNotFound'.print(); },
            StarknetError::ClassHashNotFound => { 'ClassHashNotFound'.print(); },
            StarknetError::ClassAlreadyDeclared => { 'ClassAlreadyDeclared'.print(); },
            StarknetError::InsufficientMaxFee => { 'InsufficientMaxFee'.print(); },
            StarknetError::InsufficientAccountBalance => { 'InsufficientAccountBalance'.print(); },
            StarknetError::ContractError => { 'ContractError'.print(); },
            StarknetError::InvalidTransactionNonce => { 'InvalidTransactionNonce'.print(); },
            StarknetError::ContractAddressUnavailableForDeployment => { 'AddrUnavailableForDeployment'.print(); },
            StarknetError::ClassNotDeclared => { 'ClassNotDeclared'.print(); },
            StarknetError::TransactionReverted => { 'TransactionReverted'.print(); },
        }
    }
}

#[derive(Copy, Drop, Serde, PartialEq)]
enum RPCError {
    UnknownError,
    RateLimited,
    StarknetError: StarknetError,
}

impl RPCErrorTrait of PrintTrait<RPCError> {
    #[inline(always)]
    fn print(self: RPCError) {
        match self {
            RPCError::UnknownError => { 'RPCUnknownError'.print(); },
            RPCError::RateLimited => { 'RateLimited'.print(); },
            RPCError::StarknetError(err) => { err.print(); },
        }
    }
}

#[derive(Copy, Drop, Serde, PartialEq)]
enum ScriptCommandError {
    SNCastError,
    ContractArtifactsNotFound,
    RPCError: RPCError,
}

impl ScriptCommandErrorTrait of PrintTrait<ScriptCommandError> {
    #[inline(always)]
    fn print(self: ScriptCommandError) {
        match self {
            ScriptCommandError::SNCastError => { 'SNCastError'.print(); },
            ScriptCommandError::ContractArtifactsNotFound => { 'ContractArtifactsNotFound'.print(); },
            ScriptCommandError::RPCError(err) => { err.print(); },
        }
    }
}

#[derive(Drop, Clone)]
struct CallResult {
    data: Array::<felt252>,
}

fn call(
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

#[derive(Drop, Clone, Serde)]
struct DeclareResult {
    class_hash: ClassHash,
    transaction_hash: felt252,
}

fn declare(
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

#[derive(Drop, Clone, Serde)]
struct DeployResult {
    contract_address: ContractAddress,
    transaction_hash: felt252,
}

fn deploy(
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

#[derive(Drop, Clone)]
struct InvokeResult {
    transaction_hash: felt252,
}

fn invoke(
    contract_address: ContractAddress,
    entry_point_selector: felt252,
    calldata: Array::<felt252>,
    max_fee: Option<felt252>,
    nonce: Option<felt252>
) -> InvokeResult {
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

    let buf = cheatcode::<'invoke'>(inputs.span());

    let transaction_hash = *buf[0];

    InvokeResult { transaction_hash }
}

fn get_nonce(block_tag: felt252) -> felt252 {
    let inputs = array![block_tag];
    let buf = cheatcode::<'get_nonce'>(inputs.span());
    *buf[0]
}
