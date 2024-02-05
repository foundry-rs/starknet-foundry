use crate::assert_success;
use crate::common::call_contract;
use crate::common::state::{build_runtime_state, create_runtime_states};
use crate::common::{deploy_contract, felt_selector_from_name, state::create_cached_state};

#[test]
fn segment_arena_simple() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut runtime_state_raw) = create_runtime_states(&mut cached_state);
    let mut runtime_state = build_runtime_state(&mut runtime_state_raw);
    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut runtime_state,
        "SegmentArenaUser",
        &[],
    );
    let selector = felt_selector_from_name("interface_function");
    let output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success!(output, &[]);
}
