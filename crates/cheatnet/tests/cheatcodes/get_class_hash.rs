use crate::common::call_contract;
use crate::common::state::create_cached_state;
use crate::common::{felt_selector_from_name, get_contracts, state::create_cheatnet_state};
use cairo_felt::Felt252;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::deploy::deploy;
use conversions::felt252::FromShortString;
use conversions::IntoConv;

#[test]
fn get_class_hash_simple() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contracts = get_contracts();
    let contract_name = Felt252::from_short_string("HelloStarknet").unwrap();
    let class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();
    let contract_address = deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[])
        .unwrap()
        .contract_address;

    assert_eq!(
        class_hash,
        blockifier_state.get_class_hash(contract_address).unwrap()
    );
}

#[test]
fn get_class_hash_upgrade() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contracts = get_contracts();
    let contract_name = Felt252::from_short_string("GetClassHashCheckerUpg").unwrap();
    let class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();
    let contract_address = deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[])
        .unwrap()
        .contract_address;

    assert_eq!(
        class_hash,
        blockifier_state.get_class_hash(contract_address).unwrap()
    );

    let contract_name = Felt252::from_short_string("HelloStarknet").unwrap();
    let hello_starknet_class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();

    let selector = felt_selector_from_name("upgrade");
    call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[hello_starknet_class_hash.into_()],
    )
    .unwrap();

    assert_eq!(
        hello_starknet_class_hash,
        blockifier_state.get_class_hash(contract_address).unwrap()
    );
}
