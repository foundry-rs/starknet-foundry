use crate::common::{felt_selector_from_name, get_contracts, state::create_cheatnet_state};
use cheatnet::rpc::call_contract;
use conversions::StarknetConversions;

#[test]
fn get_class_hash_simple() {
    let mut state = create_cheatnet_state();

    let contracts = get_contracts();
    let contract_name = "HelloStarknet".to_owned().to_felt252();
    let class_hash = state.declare(&contract_name, &contracts).unwrap();
    let contract_address = state.deploy(&class_hash, &[]).unwrap();

    assert_eq!(class_hash, state.get_class_hash(contract_address).unwrap());
}

#[test]
fn get_class_hash_upgrade() {
    let mut state = create_cheatnet_state();

    let contracts = get_contracts();
    let contract_name = "GetClassHashCheckerUpg".to_owned().to_felt252();
    let class_hash = state.declare(&contract_name, &contracts).unwrap();
    let contract_address = state.deploy(&class_hash, &[]).unwrap();

    assert_eq!(class_hash, state.get_class_hash(contract_address).unwrap());

    let contract_name = "HelloStarknet".to_owned().to_felt252();
    let hello_starknet_class_hash = state.declare(&contract_name, &contracts).unwrap();

    let selector = felt_selector_from_name("upgrade");
    call_contract(
        &contract_address,
        &selector,
        &[hello_starknet_class_hash.to_felt252()],
        &mut state,
    )
    .unwrap();

    assert_eq!(
        hello_starknet_class_hash,
        state.get_class_hash(contract_address).unwrap()
    );
}
