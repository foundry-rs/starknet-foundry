use crate::e2e::common::runner::{setup_package, test_runner};
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};

#[test]
fn test_does_not_work_with_exact_flag() {
    let temp = setup_package("simple_package");
    let output = test_runner(&temp)
        .args(["--partition", "3/3", "--workspace", "--exact"])
        .assert()
        .code(2);

    assert_stderr_contains(
        output,
        indoc! {r"
        error: the argument '--partition <INDEX/TOTAL>' cannot be used with '--exact'
    "},
    );
}

#[test]
fn test_whole_workspace_partition_1_2() {
    let temp = setup_package("partitioning");
    let output = test_runner(&temp)
        .args(["--partition", "1/2", "--workspace"])
        .assert()
        .code(0);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]

        Running partition run: 1/2

        Collected 2 test(s) from package_a package
        Running 1 test(s) from tests/
        [PASS] package_a_integrationtest::tests::test_c ([..])
        Running 1 test(s) from src/
        [PASS] package_a::tests::test_a ([..])
        Tests: 2 passed, 0 failed, 0 ignored, 0 filtered out, 2 skipped


        Collected 2 test(s) from package_b package
        Running 1 test(s) from src/
        [PASS] package_b::tests::test_e ([..])
        Running 1 test(s) from tests/
        [PASS] package_b_integrationtest::tests::test_g ([..])
        Tests: 2 passed, 0 failed, 0 ignored, 0 filtered out, 2 skipped


        Collected 3 test(s) from partitioning package
        Running 2 test(s) from tests/
        [PASS] partitioning_integrationtest::tests::test_k ([..])
        [PASS] partitioning_integrationtest::tests::test_m ([..])
        Running 1 test(s) from src/
        [PASS] partitioning::tests::test_i ([..])
        Tests: 3 passed, 0 failed, 0 ignored, 0 filtered out, 2 skipped


        Tests summary: 7 passed, 0 failed, 0 ignored, 0 filtered out, 6 skipped
        Finished partition run: 1/2
    "},
    );
}

#[test]
fn test_whole_workspace_partition_2_2() {
    let temp = setup_package("partitioning");
    let output = test_runner(&temp)
        .args(["--partition", "2/2", "--workspace"])
        .assert()
        .code(1);

    assert_stdout_contains(
        output,
        indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]

        Running partition run: 2/2

        Collected 2 test(s) from package_a package
        Running 1 test(s) from tests/
        [PASS] package_a_integrationtest::tests::test_d ([..])
        Running 1 test(s) from src/
        [IGNORE] package_a::tests::test_b
        Tests: 1 passed, 0 failed, 1 ignored, 0 filtered out, 2 skipped


        Collected 2 test(s) from package_b package
        Running 1 test(s) from src/
        [PASS] package_b::tests::test_f ([..])
        Running 1 test(s) from tests/
        [FAIL] package_b_integrationtest::tests::test_h

        Failure data:
            "assertion failed: `1 + 1 == 3`."

        Tests: 1 passed, 1 failed, 0 ignored, 0 filtered out, 2 skipped


        Collected 2 test(s) from partitioning package
        Running 1 test(s) from tests/
        [PASS] partitioning_integrationtest::tests::test_l ([..])
        Running 1 test(s) from src/
        [PASS] partitioning::tests::test_j ([..])
        Tests: 2 passed, 0 failed, 0 ignored, 0 filtered out, 3 skipped

        Failures:
            package_b_integrationtest::tests::test_h

        Tests summary: 4 passed, 1 failed, 1 ignored, 0 filtered out, 7 skipped
        Finished partition run: 2/2
    "#},
    );
}

#[test]
fn test_whole_workspace_partition_1_3() {
    let temp = setup_package("partitioning");
    let output = test_runner(&temp)
        .args(["--partition", "1/3", "--workspace"])
        .assert()
        .code(0);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]

        Running partition run: 1/3

        Collected 2 test(s) from package_a package
        Running 1 test(s) from src/
        [PASS] package_a::tests::test_a ([..])
        Running 1 test(s) from tests/
        [PASS] package_a_integrationtest::tests::test_d ([..])
        Tests: 2 passed, 0 failed, 0 ignored, 0 filtered out, 2 skipped


        Collected 1 test(s) from package_b package
        Running 0 test(s) from src/
        Running 1 test(s) from tests/
        [PASS] package_b_integrationtest::tests::test_g ([..])
        Tests: 1 passed, 0 failed, 0 ignored, 0 filtered out, 3 skipped


        Collected 2 test(s) from partitioning package
        Running 1 test(s) from src/
        [PASS] partitioning::tests::test_j ([..])
        Running 1 test(s) from tests/
        [PASS] partitioning_integrationtest::tests::test_m ([..])
        Tests: 2 passed, 0 failed, 0 ignored, 0 filtered out, 3 skipped


        Tests summary: 5 passed, 0 failed, 0 ignored, 0 filtered out, 8 skipped
        Finished partition run: 1/3
    "},
    );
}

#[test]
fn test_whole_workspace_partition_2_3() {
    let temp = setup_package("partitioning");
    let output = test_runner(&temp)
        .args(["--partition", "2/3", "--workspace"])
        .assert()
        .code(1);

    assert_stdout_contains(
        output,
        indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]

        Running partition run: 2/3

        Collected 1 test(s) from package_a package
        Running 0 test(s) from tests/
        Running 1 test(s) from src/
        [IGNORE] package_a::tests::test_b
        Tests: 0 passed, 0 failed, 1 ignored, 0 filtered out, 3 skipped


        Collected 2 test(s) from package_b package
        Running 1 test(s) from tests/
        [FAIL] package_b_integrationtest::tests::test_h

        Failure data:
            "assertion failed: `1 + 1 == 3`."

        Running 1 test(s) from src/
        [PASS] package_b::tests::test_e ([..])
        Tests: 1 passed, 1 failed, 0 ignored, 0 filtered out, 2 skipped


        Collected 1 test(s) from partitioning package
        Running 1 test(s) from tests/
        [PASS] partitioning_integrationtest::tests::test_k ([..])
        Running 0 test(s) from src/
        Tests: 1 passed, 0 failed, 0 ignored, 0 filtered out, 4 skipped

        Failures:
            package_b_integrationtest::tests::test_h

        Tests summary: 2 passed, 1 failed, 1 ignored, 0 filtered out, 9 skipped
        Finished partition run: 2/3
    "#},
    );
}

#[test]
fn test_whole_workspace_partition_3_3() {
    let temp = setup_package("partitioning");
    let output = test_runner(&temp)
        .args(["--partition", "3/3", "--workspace"])
        .assert()
        .code(0);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]

        Running partition run: 3/3

        Collected 1 test(s) from package_a package
        Running 1 test(s) from tests/
        [PASS] package_a_integrationtest::tests::test_c ([..])
        Running 0 test(s) from src/
        Tests: 1 passed, 0 failed, 0 ignored, 0 filtered out, 3 skipped


        Collected 1 test(s) from package_b package
        Running 1 test(s) from src/
        [PASS] package_b::tests::test_f ([..])
        Running 0 test(s) from tests/
        Tests: 1 passed, 0 failed, 0 ignored, 0 filtered out, 3 skipped


        Collected 2 test(s) from partitioning package
        Running 1 test(s) from tests/
        [PASS] partitioning_integrationtest::tests::test_l ([..])
        Running 1 test(s) from src/
        [PASS] partitioning::tests::test_i ([..])
        Tests: 2 passed, 0 failed, 0 ignored, 0 filtered out, 3 skipped


        Tests summary: 4 passed, 0 failed, 0 ignored, 0 filtered out, 9 skipped
        Finished partition run: 3/3
    "},
    );
}

#[test]
fn test_works_with_name_filter() {
    let temp = setup_package("partitioning");
    let output = test_runner(&temp)
        .args(["--partition", "1/3", "--workspace", "test_a"])
        .assert()
        .code(0);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]

        Running partition run: 1/3

        Collected 1 test(s) from package_a package
        Running 0 test(s) from tests/
        Running 1 test(s) from src/
        [PASS] package_a::tests::test_a ([..])
        Tests: 1 passed, 0 failed, 0 ignored, 1 filtered out, 0 skipped


        Collected 0 test(s) from package_b package
        Running 0 test(s) from src/
        Running 0 test(s) from tests/
        Tests: 0 passed, 0 failed, 0 ignored, 1 filtered out, 0 skipped


        Collected 0 test(s) from partitioning package
        Running 0 test(s) from tests/
        Running 0 test(s) from src/
        Tests: 0 passed, 0 failed, 0 ignored, 2 filtered out, 0 skipped


        Tests summary: 1 passed, 0 failed, 0 ignored, 4 filtered out, 0 skipped
        Finished partition run: 1/3
    "},
    );
}

#[cfg(not(feature = "cairo-native"))]
#[test]
fn test_works_with_coverage() {
    let temp = setup_package("partitioning");
    test_runner(&temp)
        .args(["--partition", "1/2", "--workspace", "--coverage"])
        .assert()
        .success();

    assert!(temp.join("coverage/coverage.lcov").is_file());
}
