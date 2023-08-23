use crate::common::{get_contracts, state::create_cheatnet_state};
use cheatnet::{
    conversions::{class_hash_to_felt, felt_from_short_string, felt_selector_from_name},
    rpc::call_contract,
};

#[test]
fn get_class_hash_simple() {
    let mut state = create_cheatnet_state();

    let contracts = get_contracts();
    let contract_name = felt_from_short_string("HelloStarknet");
    let class_hash = state.declare(&contract_name, &contracts).unwrap();
    let contract_address = state.deploy(&class_hash, vec![].as_slice()).unwrap();

    assert_eq!(class_hash, state.get_class_hash(contract_address).unwrap());
}

#[test]
fn get_class_hash_upgrade() {
    let mut state = create_cheatnet_state();

    let contracts = get_contracts();
    let contract_name = felt_from_short_string("GetClassHashCheckerUpg");
    let class_hash = state.declare(&contract_name, &contracts).unwrap();
    let contract_address = state.deploy(&class_hash, vec![].as_slice()).unwrap();

    assert_eq!(class_hash, state.get_class_hash(contract_address).unwrap());

    let contract_name = felt_from_short_string("HelloStarknet");
    let hello_starknet_class_hash = state.declare(&contract_name, &contracts).unwrap();

    let selector = felt_selector_from_name("upgrade");
    call_contract(
        &contract_address,
        &selector,
        vec![class_hash_to_felt(hello_starknet_class_hash)].as_slice(),
        &mut state,
    )
    .unwrap();

    assert_eq!(
        hello_starknet_class_hash,
        state.get_class_hash(contract_address).unwrap()
    );
}
