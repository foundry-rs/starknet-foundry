use assert_fs::fixture::PathChild;
use indoc::indoc;

use crate::e2e::common::runner::{runner, setup_package};

#[test]
#[allow(clippy::too_many_lines)]
fn file_reading() {
    let temp = setup_package("file_reading");

    let expected = indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 23 test(s) from file_reading package
        Running 9 test(s) from src/
        [PASS] file_reading::valid_content_and_same_content_no_matter_whitespaces
        [PASS] file_reading::serialization
        [PASS] file_reading::valid_content_different_folder
        [FAIL] file_reading::non_existent
        
        Failure data:
            Got an exception while executing a hint:
            No such file or directory (os error 2)
        
        [FAIL] file_reading::invalid_quotes
        
        Failure data:
            Got an exception while executing a hint:
            Failed to parse data/invalid_quotes.txt file
        
        [FAIL] file_reading::negative_number
        
        Failure data:
            Got an exception while executing a hint:
            Failed to parse data/negative_number.txt file
        
        [FAIL] file_reading::non_ascii
        
        Failure data:
            Got an exception while executing a hint:
            Failed to parse data/non_ascii.txt file
        
        [FAIL] file_reading::not_number_without_quotes
        
        Failure data:
            Got an exception while executing a hint:
            Failed to parse data/nan_without_quotes.txt file
        
        [FAIL] file_reading::too_large_number
        
        Failure data:
            Got an exception while executing a hint:
            Failed to parse data/too_large_number.txt file
        
        Running 14 test(s) from tests/
        [PASS] tests::test::valid_content_and_same_content_no_matter_whitespaces
        [PASS] tests::test::serialization
        [PASS] tests::test::json_serialization
        [FAIL] tests::test::invalid_json
        
        Failure data:
            Got an exception while executing a hint:
            Parse JSON error: invalid type: integer `231232`, expected a map at line 1 column 6 , in file data/json/invalid.json
        
        [PASS] tests::test::json_with_array
        [PASS] tests::test::json_deserialization
        [PASS] tests::test::valid_content_different_folder
        [FAIL] tests::test::non_existent
        
        Failure data:
            Got an exception while executing a hint:
            No such file or directory (os error 2)
        
        [FAIL] tests::test::json_non_existent

        Failure data:
            Got an exception while executing a hint:
            No such file or directory (os error 2)
        
        [FAIL] tests::test::invalid_quotes
        
        Failure data:
            Got an exception while executing a hint:
            Failed to parse data/invalid_quotes.txt file
        
        [FAIL] tests::test::negative_number
        
        Failure data:
            Got an exception while executing a hint:
            Failed to parse data/negative_number.txt file
        
        [FAIL] tests::test::non_ascii
        
        Failure data:
            Got an exception while executing a hint:
            Failed to parse data/non_ascii.txt file
        
        [FAIL] tests::test::not_number_without_quotes
        
        Failure data:
            Got an exception while executing a hint:
            Failed to parse data/nan_without_quotes.txt file
        
        [FAIL] tests::test::too_large_number
        
        Failure data:
            Got an exception while executing a hint:
            Failed to parse data/too_large_number.txt file
        
        Tests: 9 passed, 14 failed, 0 skipped
        
        Failures:
            file_reading::non_existent
            file_reading::invalid_quotes
            file_reading::negative_number
            file_reading::non_ascii
            file_reading::not_number_without_quotes
            file_reading::too_large_number
            tests::test::invalid_json
            tests::test::non_existent
            tests::test::json_non_existent
            tests::test::invalid_quotes
            tests::test::negative_number
            tests::test::non_ascii
            tests::test::not_number_without_quotes
            tests::test::too_large_number
    "#};

    // run from different directories to make sure cwd is always set to package directory
    let snapbox = runner();
    snapbox
        .current_dir(&temp)
        .assert()
        .code(1)
        .stdout_matches(expected);

    let snapbox = runner();
    snapbox
        .current_dir(temp.child("src"))
        .assert()
        .code(1)
        .stdout_matches(expected);

    let snapbox = runner();
    snapbox
        .current_dir(temp.child("data"))
        .assert()
        .code(1)
        .stdout_matches(expected);
}
