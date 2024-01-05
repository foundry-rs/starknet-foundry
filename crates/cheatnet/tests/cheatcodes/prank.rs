use crate::common::assertions::assert_outputs;
use crate::common::call_contract;
use crate::{
    assert_success,
    common::{
        deploy_contract, felt_selector_from_name, get_contracts, recover_data,
        state::{create_cached_state, create_cheatnet_state},
    },
};
use cairo_felt::Felt252;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::deploy::deploy;
use cheatnet::state::CheatTarget;
use conversions::felt252::FromShortString;
use conversions::IntoConv;
use starknet_api::core::ContractAddress;

#[test]
fn prank_simple() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "PrankChecker",
        &[],
    );

    cheatnet_state.start_prank(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );

    let selector = felt_selector_from_name("get_caller_address");

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn prank_with_other_syscall() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "PrankChecker",
        &[],
    );

    cheatnet_state.start_prank(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );

    let selector = felt_selector_from_name("get_caller_address_and_emit_event");

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn prank_in_constructor() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contracts = get_contracts();

    let contract_name = Felt252::from_short_string("ConstructorPrankChecker").unwrap();
    let class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();
    let precalculated_address = cheatnet_state.precalculate_address(&class_hash, &[]);

    cheatnet_state.start_prank(
        CheatTarget::One(precalculated_address),
        ContractAddress::from(123_u128),
    );

    let contract_address = deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[])
        .unwrap()
        .contract_address;

    assert_eq!(precalculated_address, contract_address);

    let selector = felt_selector_from_name("get_stored_caller_address");

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn prank_stop() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "PrankChecker",
        &[],
    );

    let selector = felt_selector_from_name("get_caller_address");

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    let old_address = recover_data(output);

    cheatnet_state.start_prank(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    let new_address = recover_data(output);
    assert_eq!(new_address, vec![Felt252::from(123)]);
    assert_ne!(old_address, new_address);

    cheatnet_state.stop_prank(CheatTarget::One(contract_address));

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();
    let changed_back_address = recover_data(output);

    assert_eq!(old_address, changed_back_address);
}

#[test]
fn prank_double() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "PrankChecker",
        &[],
    );

    let selector = felt_selector_from_name("get_caller_address");

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    let old_address = recover_data(output);

    cheatnet_state.start_prank(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );
    cheatnet_state.start_prank(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    let new_address = recover_data(output);
    assert_eq!(new_address, vec![Felt252::from(123)]);
    assert_ne!(old_address, new_address);

    cheatnet_state.stop_prank(CheatTarget::One(contract_address));

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();
    let changed_back_address = recover_data(output);

    assert_eq!(old_address, changed_back_address);
}

#[test]
fn prank_proxy() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "PrankChecker",
        &[],
    );

    let proxy_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "PrankCheckerProxy",
        &[],
    );

    let proxy_selector = felt_selector_from_name("get_prank_checkers_caller_address");
    let before_prank_output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &proxy_address,
        &proxy_selector,
        &[contract_address.into_()],
    )
    .unwrap();

    cheatnet_state.start_prank(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );

    let after_prank_output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &proxy_address,
        &proxy_selector,
        &[contract_address.into_()],
    )
    .unwrap();

    assert_success!(after_prank_output, vec![Felt252::from(123)]);

    cheatnet_state.stop_prank(CheatTarget::One(contract_address));

    let after_prank_cancellation_output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &proxy_address,
        &proxy_selector,
        &[contract_address.into_()],
    )
    .unwrap();

    assert_outputs(before_prank_output, after_prank_cancellation_output);
}

#[test]
fn prank_library_call() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contracts = get_contracts();
    let contract_name = Felt252::from_short_string("PrankChecker").unwrap();
    let class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();

    let lib_call_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "PrankCheckerLibCall",
        &[],
    );

    let lib_call_selector = felt_selector_from_name("get_caller_address_with_lib_call");
    let before_prank_output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.into_()],
    )
    .unwrap();

    cheatnet_state.start_prank(
        CheatTarget::One(lib_call_address),
        ContractAddress::from(123_u128),
    );

    let after_prank_output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.into_()],
    )
    .unwrap();

    assert_success!(after_prank_output, vec![Felt252::from(123)]);

    cheatnet_state.stop_prank(CheatTarget::One(lib_call_address));

    let after_prank_cancellation_output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.into_()],
    )
    .unwrap();

    assert_outputs(before_prank_output, after_prank_cancellation_output);
}

#[test]
fn prank_all() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "PrankChecker",
        &[],
    );

    let selector = felt_selector_from_name("get_caller_address");

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    let old_address = recover_data(output);

    cheatnet_state.start_prank(CheatTarget::All, ContractAddress::from(123_u128));

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    let new_address = recover_data(output);
    assert_eq!(new_address, vec![Felt252::from(123)]);
    assert_ne!(old_address, new_address);

    cheatnet_state.stop_prank(CheatTarget::All);

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();
    let changed_back_address = recover_data(output);

    assert_eq!(old_address, changed_back_address);
}

#[test]
fn prank_multiple() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract = Felt252::from_short_string("PrankChecker").unwrap();
    let contracts = get_contracts();
    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();

    let contract_address1 = deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[])
        .unwrap()
        .contract_address;

    let contract_address2 = deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[])
        .unwrap()
        .contract_address;

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address1,
        &felt_selector_from_name("get_caller_address"),
        &[],
    )
    .unwrap();

    let old_address1 = recover_data(output);

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address2,
        &felt_selector_from_name("get_caller_address"),
        &[],
    )
    .unwrap();

    let old_address2 = recover_data(output);

    cheatnet_state.start_prank(
        CheatTarget::Multiple(vec![contract_address1, contract_address2]),
        ContractAddress::from(123_u128),
    );

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address1,
        &felt_selector_from_name("get_caller_address"),
        &[],
    )
    .unwrap();

    let new_address1 = recover_data(output);

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address2,
        &felt_selector_from_name("get_caller_address"),
        &[],
    )
    .unwrap();

    let new_address2 = recover_data(output);

    assert_eq!(new_address1, vec![Felt252::from(123)]);
    assert_eq!(new_address2, vec![Felt252::from(123)]);

    cheatnet_state.stop_prank(CheatTarget::Multiple(vec![
        contract_address1,
        contract_address2,
    ]));

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address1,
        &felt_selector_from_name("get_caller_address"),
        &[],
    )
    .unwrap();

    let changed_back_address1 = recover_data(output);

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address2,
        &felt_selector_from_name("get_caller_address"),
        &[],
    )
    .unwrap();

    let changed_back_address2 = recover_data(output);

    assert_eq!(old_address1, changed_back_address1);
    assert_eq!(old_address2, changed_back_address2);
}

#[test]
fn prank_all_then_one() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "PrankChecker",
        &[],
    );

    let selector = felt_selector_from_name("get_caller_address");

    cheatnet_state.start_prank(CheatTarget::All, ContractAddress::from(321_u128));
    cheatnet_state.start_prank(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    assert_eq!(recover_data(output), vec![Felt252::from(123)]);
}

#[test]
fn prank_one_then_all() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "PrankChecker",
        &[],
    );

    let selector = felt_selector_from_name("get_caller_address");

    cheatnet_state.start_prank(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );
    cheatnet_state.start_prank(CheatTarget::All, ContractAddress::from(321_u128));

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    assert_eq!(recover_data(output), vec![Felt252::from(321)]);
}
