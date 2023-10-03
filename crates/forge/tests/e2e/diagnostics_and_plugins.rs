use indoc::indoc;

use crate::e2e::common::runner::{runner, setup_package};

#[test]
fn print_error_if_attributes_incorrect() {
    let temp = setup_package("diagnostics_and_plugins");
    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .assert()
        .code(2)
        .stderr_matches(indoc!
        {r#"
            error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
             --> lib.cairo:2:7
            #[fork(url: "https://lib.com")]
                  ^**********************^
            
            error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
             --> lib.cairo:2:7
            #[fork(url: "https://lib.com")]
                  ^**********************^
            
            error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
             --> test_fork.cairo:2:7
            #[fork(url: "https://test.com")]
                  ^***********************^
            
            error: Plugin diagnostic: Failed to parse the URL: relative URL without a base
             --> test_fork.cairo:8:7
            #[fork(url: "unparsable_url", block_id: BlockId::Number(1))]
                  ^***************************************************^
            
            error: Plugin diagnostic: Expected fuzzer config must be of the form `runs: <u32>, seed: <u64>`
             --> test_fuzzer.cairo:2:9
            #[fuzzer(test: 10)]
                    ^********^
            
            error: Plugin diagnostic: Expected fuzzer config must be of the form `runs: <u32>, seed: <u64>`
             --> test_fuzzer.cairo:8:9
            #[fuzzer()]
                    ^^
            
            error: Plugin diagnostic: Expected panic must be of the form `expected = <tuple of felts>`.
             --> test_should_panic.cairo:2:15
            #[should_panic(url: "https://test.com")]
                          ^***********************^

    "#});
}
