use cheatnet::rpc::{CallContractOutput, CallContractResult};
#[allow(clippy::module_name_repetitions)]
#[macro_export]
macro_rules! assert_success {
    ($call_contract_output:expr,$expected_data:expr) => {
        assert!(
            matches!(
                $call_contract_output.result,
                cheatnet::rpc::CallContractResult::Success { ret_data, .. }
                if ret_data == $expected_data,
            )
        )
    };
}

#[allow(clippy::module_name_repetitions)]
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

#[allow(clippy::module_name_repetitions)]
#[macro_export]
macro_rules! assert_error {
    ($call_contract_output:expr,$expected_data:expr) => {
        assert!(
            matches!(
                $call_contract_output.result,
                cheatnet::rpc::CallContractResult::Failure(
                    cheatnet::rpc::CallContractFailure::Error { msg, .. }
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
