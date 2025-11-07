use crate::e2e::common::runner::{
    BASE_FILE_PATTERNS, Package, setup_package_with_file_patterns, test_runner,
};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;

#[cfg_attr(
    not(feature = "run_test_for_scarb_since_2_13_1"),
    ignore = "Skipping test because feature skip_test_for_scarb_2_13 enabled"
)]
#[test]
fn wasm() {
    let temp = setup_package_with_file_patterns(
        Package::Name("wasm_oracles".to_string()),
        &[BASE_FILE_PATTERNS, &["*.wasm"]].concat(),
    );

    let output = test_runner(&temp)
        // Output of oracle is different depending on the env, and Intellij sets it automatically
        .env_remove("RUST_BACKTRACE")
        .arg("--experimental-oracles")
        .assert()
        .code(1);

    assert_stdout_contains(
        output,
        indoc! {r#"
    [..]Compiling[..]
    [..]Finished[..]

    Collected 5 test(s) from oracles package
    Running 5 test(s) from tests/
    [PASS] oracles_integrationtest::test::err ([..])
    [PASS] oracles_integrationtest::test::add ([..])
    [PASS] oracles_integrationtest::test::panic ([..])
    [FAIL] oracles_integrationtest::test::unexpected_panic

    Failure data:
        0x526573756c743a3a756e77726170206661696c65642e ('Result::unwrap failed.')
    [FAIL] oracles_integrationtest::test::panic_contents

    Failure data:
        "error while executing at wasm backtrace:
           [..]
           [..] wasm_oracle.wasm!panic

        Caused by:
            wasm trap: wasm `unreachable` instruction executed"

    Running 0 test(s) from src/
    Tests: 3 passed, 2 failed, 0 ignored, 0 filtered out
    "#},
    );
}
