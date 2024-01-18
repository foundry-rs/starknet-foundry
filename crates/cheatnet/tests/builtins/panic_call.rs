use crate::common::call_contract;
use crate::common::state::create_cheatnet_state;
use crate::common::{deploy_contract, felt_selector_from_name, state::create_cached_state};
use crate::{assert_error, assert_panic};
use cairo_felt::Felt252;
use conversions::felt252::FromShortString;
use num_traits::Bounded;

#[test]
fn call_contract_error() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address =
        deploy_contract(&mut blockifier_state, &mut cheatnet_state, "PanicCall", &[]);

    let selector = felt_selector_from_name("panic_call");

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[Felt252::from(420)],
    )
    .unwrap();

    assert_error!(output, "0x496e70757420746f6f206c6f6e6720666f7220617267756d656e7473 ('Input too long for arguments')");
}

#[test]
fn call_contract_panic() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address =
        deploy_contract(&mut blockifier_state, &mut cheatnet_state, "PanicCall", &[]);

    let selector = felt_selector_from_name("panic_call");

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

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
