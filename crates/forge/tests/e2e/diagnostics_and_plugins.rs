use super::common::runner::{setup_package_with_file_patterns, test_runner, BASE_FILE_PATTERNS};
use indoc::formatdoc;
use shared::test_utils::node_url::node_rpc_url;
use shared::test_utils::output_assert::assert_stderr_contains;

#[test]
fn print_error_if_attributes_incorrect() {
    let node_rpc_url = node_rpc_url().unwrap();
    let mock_tests_dir =
        setup_package_with_file_patterns("diagnostics_and_plugins", BASE_FILE_PATTERNS);
    let output = test_runner(&mock_tests_dir).assert().code(2);

    assert_stderr_contains(
        output,
        formatdoc! {r#"
        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> [..]/src/lib.cairo[..]
            #[fork(url: "https://lib.com")]

        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> [..]/src/lib.cairo[..]
            #[fork(url: "https://lib.com")]

        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> [..]/tests/test_fork.cairo[..]
        #[fork(url: "https://test.com")]

        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> [..]/tests/test_fork.cairo[..]
        #[fork(url: "{node_rpc_url}", block_id: BlockId::Number(Latest))]

        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> [..]/tests/test_fork.cairo[..]
        #[fork(url: "{node_rpc_url}", block_id: BlockId::Number(19446744073709551615))]

        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> [..]/tests/test_fork.cairo[..]
        #[fork(url: "{node_rpc_url}", block_id: BlockId::Hash(Random))]

        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> [..]/tests/test_fork.cairo[..]
        #[fork(url: "{node_rpc_url}", block_id: BlockId::Hash(Latest))]

        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> [..]/tests/test_fork.cairo[..]
        #[fork(url: "{node_rpc_url}", block_id: BlockId::Tag(12345))]

        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> [..]/tests/test_fork.cairo[..]
        #[fork(url: "{node_rpc_url}", block_id: BlockId::Tag(0x12345))]

        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> [..]/tests/test_fork.cairo[..]
        #[fork(url: "{node_rpc_url}", block_id: BlockId::Tag(Random))]

        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> [..]/tests/test_fork.cairo[..]
        #[fork(url: "{node_rpc_url}", block_id: BlockId::Number(Random))]

        error: Plugin diagnostic: Expected fuzzer config must be of the form `runs: <u32>, seed: <u64>`
         --> [..]/tests/test_fuzzer.cairo[..]
        #[fuzzer(test: 10)]

        error: Plugin diagnostic: Expected fuzzer config must be of the form `runs: <u32>, seed: <u64>`
         --> [..]/tests/test_fuzzer.cairo[..]
        #[fuzzer()]

        error: Plugin diagnostic: Expected panic must be of the form `expected: <tuple of felt252s and strings>` or `expected: "some string"` or `expected: <some felt252>`.
         --> [..]/tests/test_should_panic.cairo[..]
        #[should_panic(url: "https://test.com")]

        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> [..]/tests/test_fork.cairo[..]
        #[fork(url: "{node_rpc_url}", block_id: Number(12345))]

        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> [..]/tests/test_fork.cairo[..]
        #[fork(url: "{node_rpc_url}", block_id: Hash(0x12345))]

        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> [..]/tests/test_fork.cairo[..]
        #[fork(url: "{node_rpc_url}", block_id: Tag(Latest))]

        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> [..]/tests/test_fork.cairo[..]
        #[fork(url: "{node_rpc_url}", block_id: BlockWhat::Number(12345))]

        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> [..]/tests/test_fork.cairo[..]
        #[fork(url: "{node_rpc_url}", block_id: Something::BlockId::Number(12345))]

        error: Plugin diagnostic: Expected panic must be of the form `expected: <tuple of felt252s and strings>` or `expected: "some string"` or `expected: <some felt252>`.
         --> [..]/tests/test_should_panic.cairo[..]
        #[should_panic(url: "https://test.com")]

        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> [..]/tests/test_fork.cairo[..]
        #[fork(url: "{node_rpc_url}", block_id: BlockId::Tag(xddd::d00pa::hehe::BlockTag::Latest))]

        error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
         --> [..]/tests/test_fork.cairo[..]
        #[fork(url: "{node_rpc_url}", block_id: BlockId::Tag(sumting::Latest))]

        Error: Failed to compile test artifact, for detailed information go through the logs above
    "#},
    );
}
