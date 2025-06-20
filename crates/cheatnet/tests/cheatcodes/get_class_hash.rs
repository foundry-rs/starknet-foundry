use crate::common::assertions::ClassHashAssert;
use crate::common::get_contracts;
use crate::common::state::create_cached_state;
use crate::common::{call_contract, deploy_wrapper};
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::declare;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::get_class_hash::get_class_hash;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::storage::selector_from_name;
use cheatnet::state::CheatnetState;
use conversions::IntoConv;

#[test]
fn get_class_hash_simple() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contracts_data = get_contracts();
    let class_hash = declare(&mut cached_state, "HelloStarknet", &contracts_data)
        .unwrap()
        .unwrap_success();
    let contract_address =
        deploy_wrapper(&mut cached_state, &mut cheatnet_state, &class_hash, &[]).unwrap();

    assert_eq!(
        class_hash,
        get_class_hash(&mut cached_state, contract_address).unwrap()
    );
}

#[test]
fn get_class_hash_upgrade() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contracts_data = get_contracts();
    let class_hash = declare(&mut cached_state, "GetClassHashCheckerUpg", &contracts_data)
        .unwrap()
        .unwrap_success();
    let contract_address =
        deploy_wrapper(&mut cached_state, &mut cheatnet_state, &class_hash, &[]).unwrap();

    assert_eq!(
        class_hash,
        get_class_hash(&mut cached_state, contract_address).unwrap()
    );

    let hello_starknet_class_hash = declare(&mut cached_state, "HelloStarknet", &contracts_data)
        .unwrap()
        .unwrap_success();

    let selector = selector_from_name("upgrade");
    call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[hello_starknet_class_hash.into_()],
    );

    assert_eq!(
        hello_starknet_class_hash,
        get_class_hash(&mut cached_state, contract_address).unwrap()
    );
}
