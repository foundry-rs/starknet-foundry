use crate::common::{
    get_contracts,
    state::{build_runtime_state, create_cached_state, create_runtime_states},
};
use cairo_felt::Felt252;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::deploy::deploy;
use conversions::felt252::FromShortString;

#[test]
fn precalculate_address_simple() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut runtime_state_raw) = create_runtime_states(&mut cached_state);
    let mut runtime_state = build_runtime_state(&mut runtime_state_raw);

    let contracts = get_contracts();
    let contract_name = Felt252::from_short_string("HelloStarknet").unwrap();
    let class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();

    let precalculated1 = runtime_state
        .cheatnet_state
        .precalculate_address(&class_hash, &[]);
    let actual1 = deploy(&mut blockifier_state, &mut runtime_state, &class_hash, &[]).unwrap();

    let precalculated2 = runtime_state
        .cheatnet_state
        .precalculate_address(&class_hash, &[]);
    let actual2 = deploy(&mut blockifier_state, &mut runtime_state, &class_hash, &[]).unwrap();

    assert_eq!(precalculated1, actual1);
    assert_eq!(precalculated2, actual2);
    assert_ne!(actual1, actual2);
}

#[test]
fn precalculate_address_calldata() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut runtime_state_raw) = create_runtime_states(&mut cached_state);
    let mut runtime_state = build_runtime_state(&mut runtime_state_raw);

    let contracts = get_contracts();
    let contract_name = Felt252::from_short_string("ConstructorSimple").unwrap();
    let class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();

    let calldata1 = vec![Felt252::from(123)];
    let calldata2 = vec![Felt252::from(420)];

    let precalculated1 = runtime_state
        .cheatnet_state
        .precalculate_address(&class_hash, &calldata1);
    let precalculated2 = runtime_state
        .cheatnet_state
        .precalculate_address(&class_hash, &calldata2);

    let actual1 = deploy(
        &mut blockifier_state,
        &mut runtime_state,
        &class_hash,
        &calldata1,
    )
    .unwrap();

    let precalculated2_post_deploy = runtime_state
        .cheatnet_state
        .precalculate_address(&class_hash, &calldata2);

    let actual2 = deploy(
        &mut blockifier_state,
        &mut runtime_state,
        &class_hash,
        &calldata2,
    )
    .unwrap();

    assert_eq!(precalculated1, actual1);
    assert_ne!(precalculated1, precalculated2);
    assert_ne!(precalculated2, precalculated2_post_deploy);
    assert_eq!(precalculated2_post_deploy, actual2);
}
