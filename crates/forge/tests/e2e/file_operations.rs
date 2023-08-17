use assert_fs::fixture::PathChild;
use indoc::indoc;

use crate::e2e::common::runner::{runner, setup_package};

#[test]
fn file_reading() {
    let temp = setup_package("file_reading");

    let expected = indoc! {r#"Collected 18 test(s) and 2 test file(s)
        Running 9 test(s) from file_reading package
        [PASS] file_reading::valid_content_and_same_content_no_matter_whitespaces
        [PASS] file_reading::serialization
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
        
        [PASS] file_reading::valid_content_different_folder
        Running 9 test(s) from test.cairo
        [PASS] test::valid_content_and_same_content_no_matter_whitespaces
        [PASS] test::serialization
        [FAIL] test::non_existent
        
        Failure data:
            Got an exception while executing a hint:
            No such file or directory (os error 2)
        
        [FAIL] test::invalid_quotes
        
        Failure data:
            Got an exception while executing a hint:
            Failed to parse data/invalid_quotes.txt file
        
        [FAIL] test::negative_number
        
        Failure data:
            Got an exception while executing a hint:
            Failed to parse data/negative_number.txt file
        
        [FAIL] test::non_ascii
        
        Failure data:
            Got an exception while executing a hint:
            Failed to parse data/non_ascii.txt file
        
        [FAIL] test::not_number_without_quotes
        
        Failure data:
            Got an exception while executing a hint:
            Failed to parse data/nan_without_quotes.txt file
        
        [FAIL] test::too_large_number
        
        Failure data:
            Got an exception while executing a hint:
            Failed to parse data/too_large_number.txt file
        
        [PASS] test::valid_content_different_folder
        Tests: 6 passed, 12 failed, 0 skipped
        "#};

    // run from different directories to make sure cwd is always set to package directory
    let snapbox = runner();
    snapbox
        .current_dir(&temp)
        .assert()
        .success()
        .stdout_matches(expected.clone());

    let snapbox = runner();
    snapbox
        .current_dir(temp.child("src"))
        .assert()
        .success()
        .stdout_matches(expected.clone());

    let snapbox = runner();
    snapbox
        .current_dir(temp.child("data"))
        .assert()
        .success()
        .stdout_matches(expected);
}
