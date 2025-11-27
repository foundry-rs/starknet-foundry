use crate::cheatcodes::test_environment::TestEnvironment;
use crate::common::assertions::assert_success;
use crate::common::get_contracts;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::CallResult;
use cheatnet::state::CheatSpan;
use conversions::string::TryFromHexStr;
use runtime::starknet::constants::TEST_ADDRESS;
use starknet_api::core::ContractAddress;
use starknet_types_core::felt::Felt;

#[test]
fn global_cheat_works_with_library_call_from_test() {
    let mut test_env = TestEnvironment::new();
    let contracts_data = get_contracts();

    let class_hash = test_env.declare("ExampleContract", &contracts_data);
    let cheated_caller_address = 123_u8;

    test_env
        .cheatnet_state
        .start_cheat_caller_address_global(cheated_caller_address.into());

    assert_success(
        test_env.library_call_contract(&class_hash, "get_caller_address", &[]),
        &[Felt::from(cheated_caller_address)],
    );

    assert_success(
        test_env.library_call_contract(&class_hash, "get_caller_address", &[]),
        &[Felt::from(cheated_caller_address)],
    );

    test_env.cheatnet_state.stop_cheat_caller_address_global();

    assert_success(
        test_env.library_call_contract(&class_hash, "get_caller_address", &[]),
        &[Felt::try_from_hex_str(TEST_ADDRESS).unwrap()],
    );
}

#[test]
fn cheat_with_finite_span_works_with_library_call_from_test() {
    let mut test_env = TestEnvironment::new();
    let contracts_data = get_contracts();

    let class_hash = test_env.declare("ExampleContract", &contracts_data);
    let cheated_caller_address = 123_u8;

    test_env.cheatnet_state.cheat_caller_address(
        ContractAddress::try_from_hex_str(TEST_ADDRESS).unwrap(),
        cheated_caller_address.into(),
        CheatSpan::TargetCalls(1.try_into().unwrap()),
    );

    assert_success(
        test_env.library_call_contract(&class_hash, "get_caller_address", &[]),
        &[Felt::from(cheated_caller_address)],
    );

    // Library call doesn't trigger the cheat removal because it's not a contract call
    // hence after second library call the cheated address is still in effect.
    assert_success(
        test_env.library_call_contract(&class_hash, "get_caller_address", &[]),
        &[Felt::from(cheated_caller_address)],
    );
}

#[test]
fn cheat_with_indefinite_span_works_with_library_call_from_test() {
    let mut test_env = TestEnvironment::new();
    let contracts_data = get_contracts();

    let class_hash = test_env.declare("ExampleContract", &contracts_data);
    let cheated_caller_address = 123_u8;

    test_env.cheatnet_state.cheat_caller_address(
        ContractAddress::try_from_hex_str(TEST_ADDRESS).unwrap(),
        cheated_caller_address.into(),
        CheatSpan::Indefinite,
    );

    assert_success(
        test_env.library_call_contract(&class_hash, "get_caller_address", &[]),
        &[Felt::from(cheated_caller_address)],
    );

    assert_success(
        test_env.library_call_contract(&class_hash, "get_caller_address", &[]),
        &[Felt::from(cheated_caller_address)],
    );

    test_env
        .cheatnet_state
        .stop_cheat_caller_address(ContractAddress::try_from_hex_str(TEST_ADDRESS).unwrap());

    assert_success(
        test_env.library_call_contract(&class_hash, "get_caller_address", &[]),
        &[Felt::try_from_hex_str(TEST_ADDRESS).unwrap()],
    );
}

#[test]
fn global_cheat_works_with_library_call_from_actual_contract() {
    let mut test_env = TestEnvironment::new();
    let contracts_data = get_contracts();

    let contract_address = test_env.deploy("ExampleContractLibraryCall", &[]);
    let class_hash = test_env.declare("ExampleContract", &contracts_data);

    let cheated_caller_address = 123_u8;

    let set_class_hash_result =
        test_env.call_contract(&contract_address, "set_class_hash", &[class_hash.into()]);
    assert!(matches!(set_class_hash_result, CallResult::Success { .. }));

    test_env
        .cheatnet_state
        .start_cheat_caller_address_global(cheated_caller_address.into());

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt::from(cheated_caller_address)],
    );

    test_env.cheatnet_state.stop_cheat_caller_address_global();

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt::try_from_hex_str(TEST_ADDRESS).unwrap()],
    );
}

#[test]
fn cheat_with_finite_span_works_with_library_call_from_actual_contract() {
    let mut test_env = TestEnvironment::new();
    let contracts_data = get_contracts();

    let contract_address = test_env.deploy("ExampleContractLibraryCall", &[]);
    let class_hash = test_env.declare("ExampleContract", &contracts_data);

    let set_class_hash_result =
        test_env.call_contract(&contract_address, "set_class_hash", &[class_hash.into()]);
    assert!(matches!(set_class_hash_result, CallResult::Success { .. }));

    let cheated_caller_address = 123_u8;

    test_env.cheatnet_state.cheat_caller_address(
        contract_address,
        cheated_caller_address.into(),
        CheatSpan::TargetCalls(1.try_into().unwrap()),
    );

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt::from(cheated_caller_address)],
    );

    // We made one call, so the cheat should be removed now.
    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt::try_from_hex_str(TEST_ADDRESS).unwrap()],
    );
}

#[test]
fn cheat_with_indefinite_span_works_with_library_call_from_actual_contract() {
    let mut test_env = TestEnvironment::new();
    let contracts_data = get_contracts();

    let contract_address = test_env.deploy("ExampleContractLibraryCall", &[]);
    let class_hash = test_env.declare("ExampleContract", &contracts_data);

    let set_class_hash_result =
        test_env.call_contract(&contract_address, "set_class_hash", &[class_hash.into()]);
    assert!(matches!(set_class_hash_result, CallResult::Success { .. }));

    let cheated_caller_address = 123_u8;

    test_env.cheatnet_state.cheat_caller_address(
        contract_address,
        cheated_caller_address.into(),
        CheatSpan::Indefinite,
    );

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt::from(cheated_caller_address)],
    );

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt::from(cheated_caller_address)],
    );

    test_env
        .cheatnet_state
        .stop_cheat_caller_address(contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt::try_from_hex_str(TEST_ADDRESS).unwrap()],
    );
}
