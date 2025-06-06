use super::common::runner::{BASE_FILE_PATTERNS, setup_package_with_file_patterns, test_runner};
use assert_fs::fixture::PathChild;
use indoc::formatdoc;
use shared::test_utils::output_assert::assert_stdout_contains;

#[test]
fn file_reading() {
    let temp = setup_package_with_file_patterns(
        "file_reading",
        &[BASE_FILE_PATTERNS, &["**/*.txt", "**/*.json"]].concat(),
    );

    let expected_file_error = "No such file or directory [..]";

    let expected = formatdoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        
        Collected 11 test(s) from file_reading package
        Running 0 test(s) from src/
        Running 11 test(s) from tests/
        [FAIL] file_reading_integrationtest::test::json_non_existent
        
        Failure data:
            "{}"
        
        [FAIL] file_reading_integrationtest::test::invalid_json
        
        Failure data:
            "Parse JSON error: invalid type: integer `231232`, expected a map at line 1 column 6 , in file data/json/invalid.json"
        
        [FAIL] file_reading_integrationtest::test::non_existent
        
        Failure data:
            "{}"
        
        [FAIL] file_reading_integrationtest::test::non_ascii
        
        Failure data:
            "Failed to parse data/non_ascii.txt file"
        
        [PASS] file_reading_integrationtest::test::valid_content_and_same_content_no_matter_newlines [..]
        [PASS] file_reading_integrationtest::test::serialization [..]
        [PASS] file_reading_integrationtest::test::json_with_array [..]
        [FAIL] file_reading_integrationtest::test::negative_number
            "Failed to parse data/negative_number.txt file"
        
        Failure data:
        
        [FAIL] file_reading_integrationtest::test::valid_content_different_folder
        
        Failure data:
            0x756e657870656374656420636f6e74656e74 ('unexpected content')
        
        [PASS] file_reading_integrationtest::test::json_serialization [..]
        [PASS] file_reading_integrationtest::test::json_deserialization [..]
        Tests: 5 passed, 6 failed, 0 skipped, 0 ignored, 0 filtered out
        
        Failures:
            file_reading_integrationtest::test::json_non_existent
            file_reading_integrationtest::test::invalid_json
            file_reading_integrationtest::test::non_existent
            file_reading_integrationtest::test::non_ascii
            file_reading_integrationtest::test::valid_content_different_folder
            file_reading_integrationtest::test::negative_number
    "#, expected_file_error, expected_file_error};

    // run from different directories to make sure cwd is always set to package directory
    let output = test_runner(&temp).assert().code(1);

    assert_stdout_contains(output, &expected);

    let output = test_runner(&temp)
        .current_dir(temp.child("src"))
        .assert()
        .code(1);

    assert_stdout_contains(output, &expected);

    let output = test_runner(&temp)
        .current_dir(temp.child("data"))
        .assert()
        .code(1);

    assert_stdout_contains(output, &expected);
}
