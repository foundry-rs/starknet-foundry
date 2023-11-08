use std::fs;
use std::path::Path;
use test_utils::runner::Contract;
use test_utils::running_tests::run_test_case;
use test_utils::{assert_passed, test_case};

#[test]
fn code_from_benchmarks_up_to_date() {
    let test = test_case!(
        &fs::read_to_string("tests/data/contracts/benchmarks/test_declare_deploy_interact.cairo")
            .unwrap(),
        Contract::from_code_path(
            "HelloStarknet".to_string(),
            Path::new("tests/data/contracts/benchmarks/declare_deploy_interact.cairo")
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}
