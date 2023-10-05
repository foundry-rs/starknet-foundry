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
