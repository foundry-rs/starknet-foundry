use crate::common::call_contract;
use crate::common::state::build_runtime_state;
use crate::common::{deploy_contract, felt_selector_from_name, state::create_cached_state};
use crate::{assert_error, assert_panic};
use cairo_felt::Felt252;
use cheatnet::state::CheatnetState;
use conversions::felt252::FromShortString;
use num_traits::Bounded;

#[test]
fn call_contract_error() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address = deploy_contract(&mut cached_state, &mut runtime_state, "PanicCall", &[]);

    let selector = felt_selector_from_name("panic_call");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[Felt252::from(420)],
    );

    assert_error!(output, "0x496e70757420746f6f206c6f6e6720666f7220617267756d656e7473 ('Input too long for arguments')");
}

#[test]
fn call_contract_panic() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address = deploy_contract(&mut cached_state, &mut runtime_state, "PanicCall", &[]);

    let selector = felt_selector_from_name("panic_call");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_panic!(
        output,
        vec![
            Felt252::from_short_string("shortstring").unwrap(),
            Felt252::from(0),
            Felt252::max_value(),
            Felt252::from_short_string("shortstring2").unwrap()
        ]
    );
}
