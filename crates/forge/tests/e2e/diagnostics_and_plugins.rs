use crate::assert_stderr_contains;
use indoc::formatdoc;

use crate::e2e::common::runner::{setup_package, test_runner};

#[test]
fn print_error_if_attributes_incorrect() {
    let temp = setup_package("diagnostics_and_plugins");
    let temp_dir_path = temp.path().canonicalize().unwrap();
    let temp_dir_path_str = temp_dir_path.to_str().unwrap();

    let snapbox = test_runner();

    let output = snapbox.current_dir(&temp).assert().code(2);
    assert_stderr_contains!(
        output,
        &formatdoc! {r#"
            error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
             --> {temp_dir_path_str}/src/lib.cairo:4:11
                #[fork(url: "https://lib.com")]
                      ^**********************^
            
            error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
             --> {temp_dir_path_str}/src/lib.cairo:4:11
                #[fork(url: "https://lib.com")]
                      ^**********************^
            
            error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
             --> {temp_dir_path_str}/tests/test_fork.cairo:2:7
            #[fork(url: "https://test.com")]
                  ^***********************^
            
            error: Plugin diagnostic: Expected fuzzer config must be of the form `runs: <u32>, seed: <u64>`
             --> {temp_dir_path_str}/tests/test_fuzzer.cairo:2:9
            #[fuzzer(test: 10)]
                    ^********^
            
            error: Plugin diagnostic: Expected fuzzer config must be of the form `runs: <u32>, seed: <u64>`
             --> {temp_dir_path_str}/tests/test_fuzzer.cairo:8:9
            #[fuzzer()]
                    ^^

            error: Plugin diagnostic: Expected panic must be of the form `expected = <tuple of felts>`.
             --> {temp_dir_path_str}/tests/test_should_panic.cairo:2:15
            #[should_panic(url: "https://test.com")]
                          ^***********************^

    "#}
    );
}
