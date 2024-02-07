use starknet::{testing::cheatcode, ContractAddress, ClassHash};
use core::fmt::{Debug, Display, Error, Formatter};

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

#[derive(Drop, Clone, Debug)]
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
) -> CallResult {
    let contract_address_felt: felt252 = contract_address.into();
    let mut inputs = array![contract_address_felt, function_name];

    let mut calldata_serialized = array![];
    calldata.serialize(ref calldata_serialized);

    inputs.append_span(calldata_serialized.span());

    let mut buf = cheatcode::<'call'>(inputs.span());

    let result_data: Array::<felt252> = Serde::<Array<felt252>>::deserialize(ref buf).unwrap();

    CallResult { data: result_data }
}

#[derive(Drop, Clone, Debug)]
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
) -> DeclareResult {
    let mut inputs = array![contract_name];

    let mut max_fee_serialized = array![];
    max_fee.serialize(ref max_fee_serialized);

    let mut nonce_serialized = array![];
    nonce.serialize(ref nonce_serialized);

    inputs.append_span(max_fee_serialized.span());
    inputs.append_span(nonce_serialized.span());

    let buf = cheatcode::<'declare'>(inputs.span());

    let class_hash: ClassHash = (*buf[0]).try_into().expect('Invalid class hash value');
    let transaction_hash = *buf[1];

    DeclareResult { class_hash, transaction_hash }
}

#[derive(Drop, Clone, Debug)]
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
) -> DeployResult {
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

    let buf = cheatcode::<'deploy'>(inputs.span());

    let contract_address: ContractAddress = (*buf[0])
        .try_into()
        .expect('Invalid contract address value');
    let transaction_hash = *buf[1];

    DeployResult { contract_address, transaction_hash }
}

#[derive(Drop, Clone, Debug)]
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

pub fn get_nonce(block_tag: felt252) -> felt252 {
    let inputs = array![block_tag];
    let buf = cheatcode::<'get_nonce'>(inputs.span());
    *buf[0]
}
