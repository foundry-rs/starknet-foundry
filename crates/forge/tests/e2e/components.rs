use crate::e2e::common::runner::{setup_package, test_runner};

#[test]
fn contract_components() {
    let temp = setup_package("component_macros");
    let snapbox = test_runner();
    snapbox.current_dir(&temp).assert().success();
}
