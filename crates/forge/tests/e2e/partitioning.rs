use super::common::runner::{setup_package, test_runner};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;

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


        Collected 2 test(s) from package_a package
        Running 1 test(s) from tests/
        [PASS] package_a_integrationtest::tests::test_c ([..])
        Running 1 test(s) from src/
        [PASS] package_a::tests::test_a ([..])
        Tests: 2 passed, 0 failed, 0 ignored, other filtered out


        Collected 2 test(s) from package_b package
        Running 1 test(s) from src/
        [PASS] package_b::tests::test_e ([..])
        Running 1 test(s) from tests/
        [PASS] package_b_integrationtest::tests::test_g ([..])
        Tests: 2 passed, 0 failed, 0 ignored, other filtered out


        Collected 3 test(s) from partitioning package
        Running 2 test(s) from tests/
        [PASS] partitioning_integrationtest::tests::test_k ([..])
        [PASS] partitioning_integrationtest::tests::test_m ([..])
        Running 1 test(s) from src/
        [PASS] partitioning::tests::test_i ([..])
        Tests: 3 passed, 0 failed, 0 ignored, other filtered out


        Tests summary: 7 passed, 0 failed, 0 ignored, other filtered out
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


        Collected 2 test(s) from package_a package
        Running 1 test(s) from tests/
        [PASS] package_a_integrationtest::tests::test_d ([..])
        Running 1 test(s) from src/
        [IGNORE] package_a::tests::test_b
        Tests: 1 passed, 0 failed, 1 ignored, other filtered out


        Collected 2 test(s) from package_b package
        Running 1 test(s) from src/
        [PASS] package_b::tests::test_f ([..])
        Running 1 test(s) from tests/
        [FAIL] package_b_integrationtest::tests::test_h

        Failure data:
            "assertion failed: `1 + 1 == 3`."

        Tests: 1 passed, 1 failed, 0 ignored, other filtered out


        Collected 2 test(s) from partitioning package
        Running 1 test(s) from tests/
        [PASS] partitioning_integrationtest::tests::test_l ([..])
        Running 1 test(s) from src/
        [PASS] partitioning::tests::test_j ([..])
        Tests: 2 passed, 0 failed, 0 ignored, other filtered out

        Failures:
            package_b_integrationtest::tests::test_h

        Tests summary: 4 passed, 1 failed, 1 ignored, other filtered out
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


        Collected 2 test(s) from package_a package
        Running 1 test(s) from tests/
        [PASS] package_a_integrationtest::tests::test_d ([..])
        Running 1 test(s) from src/
        [PASS] package_a::tests::test_a ([..])
        Tests: 2 passed, 0 failed, 0 ignored, other filtered out


        Collected 2 test(s) from package_b package
        Running 1 test(s) from src/
        Running 1 test(s) from tests/
        [PASS] package_b_integrationtest::tests::test_g ([..])
        Tests: 1 passed, 0 failed, 0 ignored, other filtered out


        Collected 2 test(s) from partitioning package
        Running 1 test(s) from tests/
        [PASS] partitioning_integrationtest::tests::test_m ([..])
        Running 1 test(s) from src/
        [PASS] partitioning::tests::test_j ([..])
        Tests: 2 passed, 0 failed, 0 ignored, other filtered out


        Tests summary: 5 passed, 0 failed, 0 ignored, other filtered out
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


        Collected 2 test(s) from package_a package
        Running 1 test(s) from tests/
        Running 1 test(s) from src/
        [IGNORE] package_a::tests::test_b
        Tests: 0 passed, 0 failed, 1 ignored, other filtered out


        Collected 2 test(s) from package_b package
        Running 1 test(s) from tests/
        [FAIL] package_b_integrationtest::tests::test_h

        Failure data:
            "assertion failed: `1 + 1 == 3`."

        Running 1 test(s) from src/
        [PASS] package_b::tests::test_e ([..])
        Tests: 1 passed, 1 failed, 0 ignored, other filtered out


        Collected 2 test(s) from partitioning package
        Running 1 test(s) from tests/
        [PASS] partitioning_integrationtest::tests::test_k ([..])
        Running 1 test(s) from src/
        Tests: 1 passed, 0 failed, 0 ignored, other filtered out

        Failures:
            package_b_integrationtest::tests::test_h

        Tests summary: 2 passed, 1 failed, 1 ignored, other filtered out
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


        Collected 0 test(s) from package_a package
        Running 0 test(s) from src/
        Running 0 test(s) from tests/
        [PASS] package_a_integrationtest::tests::test_c ([..])
        Tests: 1 passed, 0 failed, 0 ignored, other filtered out


        Collected 0 test(s) from package_b package
        Running 0 test(s) from tests/
        Running 0 test(s) from src/
        [PASS] package_b::tests::test_f ([..])
        Tests: 1 passed, 0 failed, 0 ignored, other filtered out


        Collected 1 test(s) from partitioning package
        Running 1 test(s) from tests/
        [PASS] partitioning_integrationtest::tests::test_l ([..])
        Running 0 test(s) from src/
        [PASS] partitioning::tests::test_i ([..])
        Tests: 2 passed, 0 failed, 0 ignored, other filtered out


        Tests summary: 4 passed, 0 failed, 0 ignored, 1other filtered out
        Finished partition run: 3/3
    "},
    );
}

#[test]
fn test_does_not_work_with_exact_flag() {
    let temp = setup_package("partitioning");
    let output = test_runner(&temp)
        .args(["--partition", "3/3", "--workspace"])
        .assert()
        .code(2);

    assert_stdout_contains(
        output,
        indoc! {r"
        error: the argument '--partition <PARTITION>' cannot be used with '--exact'
    "},
    );
}
