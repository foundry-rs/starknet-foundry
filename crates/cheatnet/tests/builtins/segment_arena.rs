use crate::assert_success;
use crate::common::call_contract;
use crate::common::state::create_cheatnet_state;
use crate::common::{deploy_contract, felt_selector_from_name, state::create_cached_state};

#[test]
fn segment_arena_simple() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);
    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "SegmentArenaUser",
        &[],
    );
    let selector = felt_selector_from_name("interface_function");
    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    assert_success!(output, &[]);
}
