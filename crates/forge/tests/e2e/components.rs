use super::common::runner::{setup_package, test_runner};

#[test]
fn contract_components() {
    let temp = setup_package("component_macros");

    test_runner(&temp).assert().success();
}
