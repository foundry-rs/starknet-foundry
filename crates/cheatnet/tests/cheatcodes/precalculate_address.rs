use crate::common::{get_contracts, state::create_cheatnet_state};
use cairo_felt::Felt252;
use conversions::StarknetConversions;

#[test]
fn precalculate_address_simple() {
    let mut state = create_cheatnet_state();

    let contracts = get_contracts();
    let contract_name = "HelloStarknet".to_owned().to_felt252();
    let class_hash = state.declare(&contract_name, &contracts).unwrap();

    let precalculated1 = state.precalculate_address(&class_hash, &[]);
    let actual1 = state.deploy(&class_hash, &[]).unwrap().contract_address;

    let precalculated2 = state.precalculate_address(&class_hash, &[]);
    let actual2 = state.deploy(&class_hash, &[]).unwrap().contract_address;

    assert_eq!(precalculated1, actual1);
    assert_eq!(precalculated2, actual2);
    assert_ne!(actual1, actual2);
}

#[test]
fn precalculate_address_calldata() {
    let mut state = create_cheatnet_state();

    let contracts = get_contracts();
    let contract_name = "ConstructorSimple".to_owned().to_felt252();
    let class_hash = state.declare(&contract_name, &contracts).unwrap();

    let calldata1 = vec![Felt252::from(123)];
    let calldata2 = vec![Felt252::from(420)];

    let precalculated1 = state.precalculate_address(&class_hash, &calldata1);
    let precalculated2 = state.precalculate_address(&class_hash, &calldata2);

    let actual1 = state
        .deploy(&class_hash, &calldata1)
        .unwrap()
        .contract_address;

    let precalculated2_post_deploy = state.precalculate_address(&class_hash, &calldata2);

    let actual2 = state
        .deploy(&class_hash, &calldata2)
        .unwrap()
        .contract_address;

    assert_eq!(precalculated1, actual1);
    assert_ne!(precalculated1, precalculated2);
    assert_ne!(precalculated2, precalculated2_post_deploy);
    assert_eq!(precalculated2_post_deploy, actual2);
}
