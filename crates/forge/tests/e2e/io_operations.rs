use super::common::runner::{setup_package_with_file_patterns, test_runner, BASE_FILE_PATTERNS};
use assert_fs::fixture::PathChild;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;

#[test]
#[allow(clippy::too_many_lines)]
fn file_reading() {
    let temp = setup_package_with_file_patterns(
        "file_reading",
        &[BASE_FILE_PATTERNS, &["**/*.txt", "**/*.json"]].concat(),
    );

    let expected = indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        
        Collected 11 test(s) from file_reading package
        Running 0 test(s) from src/
        Running 11 test(s) from tests/
        [FAIL] tests::test::json_non_existent
        
        Failure data:
            "No such file or directory [..]"
        
        [FAIL] tests::test::invalid_json
        
        Failure data:
            "Parse JSON error: invalid type: integer `231232`, expected a map at line 1 column 6 , in file data/json/invalid.json"
        
        [FAIL] tests::test::non_existent
        
        Failure data:
            "No such file or directory [..]"
        
        [FAIL] tests::test::non_ascii
        
        Failure data:
            "Failed to parse data/non_ascii.txt file"
        
        [PASS] tests::test::valid_content_and_same_content_no_matter_newlines [..]
        [PASS] tests::test::serialization [..]
        [PASS] tests::test::json_with_array [..]
        [FAIL] tests::test::negative_number
            "Failed to parse data/negative_number.txt file"
        
        Failure data:
        
        [FAIL] tests::test::valid_content_different_folder
        
        Failure data:
            0x756e657870656374656420636f6e74656e74 ('unexpected content')
        
        [PASS] tests::test::json_serialization [..]
        [PASS] tests::test::json_deserialization [..]
        Tests: 5 passed, 6 failed, 0 skipped, 0 ignored, 0 filtered out
        
        Failures:
            tests::test::json_non_existent
            tests::test::invalid_json
            tests::test::non_existent
            tests::test::non_ascii
            tests::test::valid_content_different_folder
            tests::test::negative_number
    "#};

    // run from different directories to make sure cwd is always set to package directory
    let output = test_runner(&temp).assert().code(1);

    assert_stdout_contains(output, expected);

    let output = test_runner(&temp)
        .current_dir(temp.child("src"))
        .assert()
        .code(1);

    assert_stdout_contains(output, expected);

    let output = test_runner(&temp)
        .current_dir(temp.child("data"))
        .assert()
        .code(1);

    assert_stdout_contains(output, expected);
}
