use crate::common::assertions::{assert_error, assert_panic};
use crate::common::call_contract;
use crate::common::{deploy_contract, felt_selector_from_name, state::create_cached_state};
use cairo_lang_utils::byte_array::BYTE_ARRAY_MAGIC;
use cairo_vm::Felt252;
use cheatnet::state::CheatnetState;
use conversions::felt252::FromShortString;
use conversions::string::TryFromHexStr;
use conversions::IntoConv;

#[test]
fn call_contract_error() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contract_address =
        deploy_contract(&mut cached_state, &mut cheatnet_state, "PanicCall", &[]);

    let selector = felt_selector_from_name("panic_call");

    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[Felt252::from(420)],
    );

    assert_error(output, "\n    0x496e70757420746f6f206c6f6e6720666f7220617267756d656e7473 ('Input too long for arguments')\n");
}

#[test]
fn call_contract_panic() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contract_address =
        deploy_contract(&mut cached_state, &mut cheatnet_state, "PanicCall", &[]);

    let selector = felt_selector_from_name("panic_call");

    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );

    assert_panic(
        output,
        &[
            Felt252::from_short_string("shortstring").unwrap(),
            Felt252::from(0),
            Felt252::MAX,
            Felt252::from_short_string("shortstring2").unwrap(),
        ],
    );
}

#[test]
fn call_proxied_contract_bytearray_panic() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let proxy = deploy_contract(
        &mut cached_state,
        &mut cheatnet_state,
        "ByteArrayPanickingContractProxy",
        &[],
    );
    let bytearray_panicking_contract = deploy_contract(
        &mut cached_state,
        &mut cheatnet_state,
        "ByteArrayPanickingContract",
        &[],
    );

    let selector = felt_selector_from_name("call_bytearray_panicking_contract");

    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &proxy,
        selector,
        &[bytearray_panicking_contract.into_()],
    );

    assert_panic(
        output,
        &[
            Felt252::try_from_hex_str(&format!("0x{BYTE_ARRAY_MAGIC}")).unwrap(),
            Felt252::from(2),
            Felt252::from_short_string("This is a very long\n and multi ").unwrap(),
            Felt252::from_short_string("line string, that will for sure").unwrap(),
            Felt252::from_short_string(" saturate the pending_word").unwrap(),
            Felt252::from(26),
        ],
    );
}
