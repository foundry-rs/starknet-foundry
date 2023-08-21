#[allow(clippy::module_name_repetitions)]
#[macro_export]
macro_rules! assert_success {
    ($call_contract_output:expr,$expected_data:expr) => {
        assert!(match $call_contract_output {
            cheatnet::rpc::CallContractOutput::Success { ret_data } => ret_data == $expected_data,
            _ => false,
        })
    };
}

#[allow(clippy::module_name_repetitions)]
#[macro_export]
macro_rules! assert_panic {
    ($call_contract_output:expr,$expected_data:expr) => {
        assert!(match $call_contract_output {
            cheatnet::rpc::CallContractOutput::Panic { panic_data } => panic_data == $expected_data,
            _ => false,
        })
    };
}
