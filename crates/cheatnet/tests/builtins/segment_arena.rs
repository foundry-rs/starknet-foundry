use crate::common::assertions::assert_success;
use crate::common::state::{build_runtime_state, create_cached_state};
use crate::common::{call_contract, deploy_contract, felt_selector_from_name};
use cheatnet::state::CheatnetState;

#[test]
fn segment_arena_simple() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut runtime_state,
        "SegmentArenaUser",
        &[],
    );
    let selector = felt_selector_from_name("interface_function");
    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success(output, &[]);
}
