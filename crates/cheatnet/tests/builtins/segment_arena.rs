use crate::common::assertions::assert_success;
use crate::common::call_contract;
use crate::common::{deploy_contract, state::create_cached_state};
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::storage::felt_selector_from_name;
use cheatnet::state::CheatnetState;

#[test]
fn segment_arena_simple() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut cheatnet_state,
        "SegmentArenaUser",
        &[],
    );
    let selector = felt_selector_from_name("interface_function");
    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );

    assert_success(output, &[]);
}
