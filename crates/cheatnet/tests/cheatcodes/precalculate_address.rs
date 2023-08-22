use crate::common::{get_contracts, state::create_cheatnet_state};
use cairo_felt::Felt252;
use cheatnet::conversions::felt_from_short_string;

#[test]
fn precalculate_address_simple() {
    let mut state = create_cheatnet_state();

    let contracts = get_contracts();
    let contract_name = felt_from_short_string("HelloStarknet");
    let class_hash = state.declare(&contract_name, &contracts).unwrap();

    let precalculated1 = state.precalculate_address(&class_hash, vec![].as_slice());
    let actual1 = state.deploy(&class_hash, vec![].as_slice()).unwrap();

    let precalculated2 = state.precalculate_address(&class_hash, vec![].as_slice());
    let actual2 = state.deploy(&class_hash, vec![].as_slice()).unwrap();

    assert_eq!(precalculated1, actual1);
    assert_eq!(precalculated2, actual2);
    assert_ne!(actual1, actual2);
}

#[test]
fn precalculate_address_calldata() {
    let mut state = create_cheatnet_state();

    let contracts = get_contracts();
    let contract_name = felt_from_short_string("ConstructorSimple");
    let class_hash = state.declare(&contract_name, &contracts).unwrap();

    let calldata1 = vec![Felt252::from(123)];
    let calldata2 = vec![Felt252::from(420)];

    let precalculated1 = state.precalculate_address(&class_hash, &calldata1);
    let precalculated2 = state.precalculate_address(&class_hash, &calldata2);

    let actual1 = state.deploy(&class_hash, &calldata1).unwrap();

    let precalculated2_post_deploy = state.precalculate_address(&class_hash, &calldata2);

    let actual2 = state.deploy(&class_hash, &calldata2).unwrap();

    assert_eq!(precalculated1, actual1);
    assert_ne!(precalculated1, precalculated2);
    assert_ne!(precalculated2, precalculated2_post_deploy);
    assert_eq!(precalculated2_post_deploy, actual2);
}
