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

    let expected = indoc! {r"

    "};

    // run from different directories to make sure cwd is always set to package directory
    let output = test_runner(&temp).assert().code(1);
    unsafe {
        println!(
            "{}",
            String::from_utf8_unchecked(output.get_output().stdout.clone())
        );
    };
    output.success();
    // assert_stdout_contains(output, expected);

    // let output = test_runner(&temp)
    //     .current_dir(temp.child("src"))
    //     .assert()
    //     .code(1);

    // assert_stdout_contains(output, expected);

    // let output = test_runner(&temp)
    //     .current_dir(temp.child("data"))
    //     .assert()
    //     .code(1);

    // assert_stdout_contains(output, expected);
}
