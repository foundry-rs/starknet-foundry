use crate::assert_stderr_contains;
use indoc::formatdoc;

use crate::e2e::common::runner::{setup_package, test_runner};

#[test]
fn print_error_if_attributes_incorrect() {
    let mock_tests_dir = setup_package("diagnostics_and_plugins");
    let mock_tests_dir_path = mock_tests_dir.path().canonicalize().unwrap();
    let mock_tests_dir_path_str = mock_tests_dir_path.to_str().unwrap();

    let snapbox = test_runner();

    let output = snapbox.current_dir(&mock_tests_dir).assert().code(2);
    assert_stderr_contains!(
        output,
        &formatdoc! {r#"
        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> {mock_tests_dir_path_str}/src/lib.cairo:4:11
            #[fork(url: "https://lib.com")]
                  ^**********************^
        
        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> {mock_tests_dir_path_str}/src/lib.cairo:4:11
            #[fork(url: "https://lib.com")]
                  ^**********************^
        
        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> {mock_tests_dir_path_str}/tests/test_fork.cairo:2:7
        #[fork(url: "https://test.com")]
              ^***********************^
        
        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> {mock_tests_dir_path_str}/tests/test_fork.cairo:8:7
        #[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: BlockId::Number(Latest))]
              ^*****************************************************************************^
        
        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> {mock_tests_dir_path_str}/tests/test_fork.cairo:14:7
        #[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: BlockId::Number(19446744073709551615))]
              ^*******************************************************************************************^
        
        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> {mock_tests_dir_path_str}/tests/test_fork.cairo:20:7
        #[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: BlockId::Hash(Random))]
              ^***************************************************************************^
        
        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> {mock_tests_dir_path_str}/tests/test_fork.cairo:26:7
        #[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: BlockId::Hash(Latest))]
              ^***************************************************************************^
        
        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> {mock_tests_dir_path_str}/tests/test_fork.cairo:32:7
        #[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: BlockId::Tag(12345))]
              ^*************************************************************************^
        
        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> {mock_tests_dir_path_str}/tests/test_fork.cairo:38:7
        #[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: BlockId::Tag(0x12345))]
              ^***************************************************************************^
        
        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> {mock_tests_dir_path_str}/tests/test_fork.cairo:44:7
        #[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: BlockId::Tag(Random))]
              ^**************************************************************************^
        
        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> {mock_tests_dir_path_str}/tests/test_fork.cairo:50:7
        #[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: BlockId::Number(Random))]
              ^*****************************************************************************^
        
        error: Plugin diagnostic: Expected fuzzer config must be of the form `runs: <u32>, seed: <u64>`
         --> {mock_tests_dir_path_str}/tests/test_fuzzer.cairo:2:9
        #[fuzzer(test: 10)]
                ^********^
        
        error: Plugin diagnostic: Expected fuzzer config must be of the form `runs: <u32>, seed: <u64>`
         --> {mock_tests_dir_path_str}/tests/test_fuzzer.cairo:8:9
        #[fuzzer()]
                ^^
        
        error: Plugin diagnostic: Expected panic must be of the form `expected: <tuple of felt252s and strings>` or `expected: "some string"` or `expected: <some felt252>`.
         --> {mock_tests_dir_path_str}/tests/test_should_panic.cairo:2:15
        #[should_panic(url: "https://test.com")]
                      ^***********************^


        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> {mock_tests_dir_path_str}/tests/test_fork.cairo:56:7
        #[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: Number(12345))]
              ^*******************************************************************^

        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> {mock_tests_dir_path_str}/tests/test_fork.cairo:62:7
        #[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: Hash(0x12345))]
              ^*******************************************************************^

        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> {mock_tests_dir_path_str}/tests/test_fork.cairo:68:7
        #[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: Tag(Latest))]
              ^*****************************************************************^

        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> {mock_tests_dir_path_str}/tests/test_fork.cairo:74:7
        #[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: BlockWhat::Number(12345))]
              ^******************************************************************************^

        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> {mock_tests_dir_path_str}/tests/test_fork.cairo:80:7
        #[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: Something::BlockId::Number(12345))]
              ^***************************************************************************************^

        error: Plugin diagnostic: Expected panic must be of the form `expected: <tuple of felt252s and strings>` or `expected: "some string"` or `expected: <some felt252>`.
         --> {mock_tests_dir_path_str}/tests/test_should_panic.cairo:2:15
        #[should_panic(url: "https://test.com")]
                      ^***********************^
        Error: Failed to compile test artifact, for detailed information go through the logs above
        
        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> {mock_tests_dir_path_str}/tests/test_fork.cairo:86:7
        #[fork(
              ^

        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> {mock_tests_dir_path_str}/tests/test_fork.cairo:95:7
        #[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: BlockId::Tag(sumting::Latest))]
              ^***********************************************************************************^
    "#}
    );
}
