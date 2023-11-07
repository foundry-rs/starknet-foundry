use crate::assert_stdout_contains;
use assert_fs::fixture::PathChild;
use indoc::indoc;

use crate::e2e::common::runner::{
    setup_package_with_file_patterns, test_runner, BASE_FILE_PATTERNS,
};

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
    
    
        Collected 23 test(s) from file_reading package
        Running 9 test(s) from src/
        [FAIL] file_reading::tests::negative_number
        
        Failure data:
            Got an exception while executing a hint: Hint Error: Failed to parse data/negative_number.txt file
        
        [FAIL] file_reading::tests::too_large_number
        
        Failure data:
            Got an exception while executing a hint: Hint Error: Failed to parse data/too_large_number.txt file
        
        [FAIL] file_reading::tests::non_ascii
        
        Failure data:
            Got an exception while executing a hint: Hint Error: Failed to parse data/non_ascii.txt file
        
        [FAIL] file_reading::tests::invalid_quotes
        
        Failure data:
            Got an exception while executing a hint: Hint Error: Failed to parse data/invalid_quotes.txt file
        
        [PASS] file_reading::tests::valid_content_different_folder
        [PASS] file_reading::tests::valid_content_and_same_content_no_matter_whitespaces
        [PASS] file_reading::tests::serialization
        [FAIL] file_reading::tests::non_existent
        
        Failure data:
            Got an exception while executing a hint: Hint Error: No such file or directory (os error 2)
        
        [FAIL] file_reading::tests::not_number_without_quotes
        
        Failure data:
            Got an exception while executing a hint: Hint Error: Failed to parse data/nan_without_quotes.txt file
        
        Running 14 test(s) from tests/
        [FAIL] tests::test::too_large_number
        
        Failure data:
            Got an exception while executing a hint: Hint Error: Failed to parse data/too_large_number.txt file
        
        [FAIL] tests::test::non_existent
        
        Failure data:
            Got an exception while executing a hint: Hint Error: No such file or directory (os error 2)
        
        [PASS] tests::test::json_with_array
        [FAIL] tests::test::invalid_json
        
        Failure data:
            Got an exception while executing a hint: Hint Error: Parse JSON error: invalid type: integer `231232`, expected a map at line 1 column 6 , in file data/json/invalid.json
        
        [PASS] tests::test::json_deserialization
        [FAIL] tests::test::non_ascii
        
        Failure data:
            Got an exception while executing a hint: Hint Error: Failed to parse data/non_ascii.txt file
        
        [PASS] tests::test::valid_content_different_folder
        [PASS] tests::test::json_serialization
        [FAIL] tests::test::invalid_quotes
        
        Failure data:
            Got an exception while executing a hint: Hint Error: Failed to parse data/invalid_quotes.txt file
        
        [FAIL] tests::test::json_non_existent
        
        Failure data:
            Got an exception while executing a hint: Hint Error: No such file or directory (os error 2)
        
        [PASS] tests::test::valid_content_and_same_content_no_matter_whitespaces
        [FAIL] tests::test::negative_number
        
        Failure data:
            Got an exception while executing a hint: Hint Error: Failed to parse data/negative_number.txt file
        
        [PASS] tests::test::serialization
        [FAIL] tests::test::not_number_without_quotes
        
        Failure data:
            Got an exception while executing a hint: Hint Error: Failed to parse data/nan_without_quotes.txt file
        
        Tests: 9 passed, 14 failed, 0 skipped, 0 ignored, 0 filtered out
        
        Failures:
            file_reading::tests::negative_number
            file_reading::tests::too_large_number
            file_reading::tests::non_ascii
            file_reading::tests::invalid_quotes
            file_reading::tests::non_existent
            file_reading::tests::not_number_without_quotes
            tests::test::too_large_number
            tests::test::non_existent
            tests::test::invalid_json
            tests::test::non_ascii
            tests::test::invalid_quotes
            tests::test::json_non_existent
            tests::test::negative_number
            tests::test::not_number_without_quotes
    "#};

    // run from different directories to make sure cwd is always set to package directory
    let snapbox = test_runner();
    let output = snapbox.current_dir(&temp).assert().code(1);
    assert_stdout_contains!(output, expected);

    let snapbox = test_runner();
    let output = snapbox.current_dir(temp.child("src")).assert().code(1);
    assert_stdout_contains!(output, expected);

    let snapbox = test_runner();
    let output = snapbox.current_dir(temp.child("data")).assert().code(1);
    assert_stdout_contains!(output, expected);
}
