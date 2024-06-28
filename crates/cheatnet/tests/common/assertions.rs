use cairo_vm::Felt252;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    CallFailure, CallResult,
};

#[inline]
pub fn assert_success(call_contract_output: CallResult, expected_data: &[Felt252]) {
    assert!(matches!(
        call_contract_output,
        CallResult::Success { ret_data, .. }
        if ret_data == expected_data,
    ));
}

#[inline]
pub fn assert_panic(call_contract_output: CallResult, expected_data: &[Felt252]) {
    assert!(matches!(
        call_contract_output,
        CallResult::Failure(
            CallFailure::Panic { panic_data, .. }
        )
        if panic_data == expected_data
    ));
}

#[inline]
pub fn assert_error(call_contract_output: CallResult, expected_data: impl Into<String>) {
    assert!(matches!(
        call_contract_output,
        CallResult::Failure(
            CallFailure::Error { msg, .. }
        )
        if msg == expected_data.into(),
    ));
}
