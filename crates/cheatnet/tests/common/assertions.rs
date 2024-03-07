use cairo_felt::Felt252;
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

#[inline]
pub fn assert_outputs(output1: CallResult, output2: CallResult) {
    let CallResult::Success {
        ret_data: before_ret_data,
    } = output1
    else {
        panic!("Unexpected failure")
    };

    let CallResult::Success {
        ret_data: after_ret_data,
    } = output2
    else {
        panic!("Unexpected failure")
    };

    assert_eq!(before_ret_data, after_ret_data);
}
