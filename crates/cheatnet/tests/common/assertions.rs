use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    CallFailure, CallResult,
};
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::DeclareResult;
use conversions::byte_array::ByteArray;
use starknet_api::core::ClassHash;
use starknet_types_core::felt::Felt;

#[inline]
pub fn assert_success(call_contract_output: CallResult, expected_data: &[Felt]) {
    assert!(matches!(
        call_contract_output,
        CallResult::Success { ret_data, .. }
        if ret_data == expected_data,
    ));
}

#[inline]
pub fn assert_panic(call_contract_output: CallResult, expected_data: &[Felt]) {
    assert!(matches!(
        call_contract_output,
        CallResult::Failure(
            CallFailure::Panic { panic_data, .. }
        )
        if panic_data == expected_data
    ));
}

#[inline]
pub fn assert_error(call_contract_output: CallResult, expected_data: impl Into<ByteArray>) {
    assert!(matches!(
        call_contract_output,
        CallResult::Failure(
            CallFailure::Error { msg, .. }
        )
        if msg == expected_data.into(),
    ));
}

pub trait ClassHashAssert {
    fn unwrap_success(self) -> ClassHash;
}

impl ClassHashAssert for DeclareResult {
    fn unwrap_success(self) -> ClassHash {
        match self {
            DeclareResult::Success(class_hash) => class_hash,
            DeclareResult::AlreadyDeclared(class_hash) => {
                panic!("Class hash {class_hash} is already declared")
            }
        }
    }
}
