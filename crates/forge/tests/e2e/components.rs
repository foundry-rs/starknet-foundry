use crate::e2e::common::runner::setup_package;

use super::common::runner::test_runner;

#[test]
fn contract_components() {
    let temp = setup_package("component_macros");

    test_runner(&temp).assert().success();
}
