use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    CallContractOutput, CallContractResult,
};
#[macro_export]
macro_rules! assert_success {
    ($call_contract_output:expr,$expected_data:expr) => {
        assert!(
            matches!(
                $call_contract_output.result,
                cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::CallContractResult::Success { ret_data, .. }
                if ret_data == $expected_data,
            )
        )
    };
}

#[macro_export]
macro_rules! assert_panic {
    ($call_contract_output:expr,$expected_data:expr) => {
        assert!(
            matches!(
                $call_contract_output.result,
                cheatnet::rpc::CallContractResult::Failure(
                    cheatnet::rpc::CallContractFailure::Panic { panic_data, .. }
                )
                if panic_data == $expected_data
            )
        );
    };
}

#[macro_export]
macro_rules! assert_error {
    ($call_contract_output:expr,$expected_data:expr) => {
        assert!(
            matches!(
                $call_contract_output.result,
                cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::CallContractResult::Failure(
                    cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::CallContractFailure::Error { msg, .. }
                )
                if msg == $expected_data,
            )
        )
    };
}

pub fn assert_outputs(output1: CallContractOutput, output2: CallContractOutput) {
    let CallContractResult::Success {
        ret_data: before_ret_data,
    } = output1.result
    else {
        panic!("Unexpected failure")
    };

    let CallContractResult::Success {
        ret_data: after_ret_data,
    } = output2.result
    else {
        panic!("Unexpected failure")
    };

    assert_eq!(before_ret_data, after_ret_data);
}
