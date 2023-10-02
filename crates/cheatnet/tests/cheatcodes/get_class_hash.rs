use crate::common::state::create_cached_state;
use crate::common::{felt_selector_from_name, get_contracts, state::create_cheatnet_state};
use cheatnet::cheatcodes::deploy::deploy;
use cheatnet::rpc::call_contract;
use conversions::StarknetConversions;

#[test]
fn get_class_hash_simple() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contracts = get_contracts();
    let contract_name = "HelloStarknet".to_owned().to_felt252();
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
    let contract_name = "GetClassHashCheckerUpg".to_owned().to_felt252();
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

    let contract_name = "HelloStarknet".to_owned().to_felt252();
    let hello_starknet_class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();

    let selector = felt_selector_from_name("upgrade");
    call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[hello_starknet_class_hash.to_felt252()],
    )
    .unwrap();

    assert_eq!(
        hello_starknet_class_hash,
        blockifier_state.get_class_hash(contract_address).unwrap()
    );
}
