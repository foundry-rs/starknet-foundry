use indoc::indoc;

use crate::e2e::common::runner::{runner, setup_package};

#[test]
fn print_error_if_attributes_incorrect() {
    let temp = setup_package("fork");
    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .arg("fork::")
        .assert()
        .stderr_matches(indoc!
        {r#"error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
        --> test_fork.cairo:2:7
       #[fork(url: "https://test.com")]
             ^***********************^
       
       error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
        --> test_fork.cairo:2:7
       #[fork(url: "https://test.com")]
             ^***********************^

       error: Plugin diagnostic: Expected fork config must be of the form `url: <double quote string>, block_id: <snforge_std::BlockId>`.
        --> test_fork.cairo:2:7
       #[fork(url: "https://test.com")]
             ^***********************^
       
    "#});
}
