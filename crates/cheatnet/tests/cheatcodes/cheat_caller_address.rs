use crate::common::assertions::assert_success;
use crate::common::get_contracts;
use cairo_lang_starknet_classes::keccak::starknet_keccak;
use cheatnet::constants::TEST_ADDRESS;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::spy_events::Event;
use cheatnet::state::CheatSpan;
use conversions::IntoConv;
use starknet_api::core::ContractAddress;
use starknet_types_core::felt::Felt;
use tempfile::TempDir;

use super::test_environment::TestEnvironment;
use crate::common::state::create_fork_cached_state_at;
use conversions::string::TryFromHexStr;

trait CheatCallerAddressTrait {
    fn cheat_caller_address(
        &mut self,
        contract_address: ContractAddress,
        new_address: u128,
        span: CheatSpan,
    );
    fn start_cheat_caller_address(&mut self, contract_address: ContractAddress, new_address: u128);
    fn stop_cheat_caller_address(&mut self, contract_address: ContractAddress);
}

impl CheatCallerAddressTrait for TestEnvironment {
    fn cheat_caller_address(
        &mut self,
        contract_address: ContractAddress,
        new_address: u128,
        span: CheatSpan,
    ) {
        self.cheatnet_state.cheat_caller_address(
            contract_address,
            ContractAddress::from(new_address),
            span,
        );
    }

    fn start_cheat_caller_address(&mut self, contract_address: ContractAddress, new_address: u128) {
        self.cheatnet_state
            .start_cheat_caller_address(contract_address, ContractAddress::from(new_address));
    }

    fn stop_cheat_caller_address(&mut self, contract_address: ContractAddress) {
        self.cheatnet_state
            .stop_cheat_caller_address(contract_address);
    }
}

#[test]
fn cheat_caller_address_simple() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatCallerAddressChecker", &[]);

    test_env.start_cheat_caller_address(contract_address, 123);

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt::from(123)],
    );
}

#[test]
fn cheat_caller_address_with_other_syscall() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatCallerAddressChecker", &[]);

    test_env.start_cheat_caller_address(contract_address, 123);

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address_and_emit_event", &[]),
        &[Felt::from(123)],
    );
}

#[test]
fn cheat_caller_address_in_constructor() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("ConstructorCheatCallerAddressChecker", &contracts_data);
    let precalculated_address = test_env.precalculate_address(&class_hash, &[]);

    test_env.start_cheat_caller_address(precalculated_address, 123);

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);
    assert_eq!(precalculated_address, contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_stored_caller_address", &[]),
        &[Felt::from(123)],
    );
}

#[test]
fn cheat_caller_address_stop() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatCallerAddressChecker", &[]);

    test_env.start_cheat_caller_address(contract_address, 123);

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt::from(123)],
    );

    test_env.stop_cheat_caller_address(contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[TryFromHexStr::try_from_hex_str(TEST_ADDRESS).unwrap()],
    );
}

#[test]
fn cheat_caller_address_double() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatCallerAddressChecker", &[]);

    test_env.start_cheat_caller_address(contract_address, 111);
    test_env.start_cheat_caller_address(contract_address, 222);

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt::from(222)],
    );

    test_env.stop_cheat_caller_address(contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[TryFromHexStr::try_from_hex_str(TEST_ADDRESS).unwrap()],
    );
}

#[test]
fn cheat_caller_address_proxy() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatCallerAddressChecker", &[]);
    let proxy_address = test_env.deploy("CheatCallerAddressCheckerProxy", &[]);

    test_env.start_cheat_caller_address(contract_address, 123);

    let selector = "get_cheated_caller_address";
    assert_success(
        test_env.call_contract(&proxy_address, selector, &[contract_address.into_()]),
        &[Felt::from(123)],
    );

    test_env.stop_cheat_caller_address(contract_address);

    assert_success(
        test_env.call_contract(&proxy_address, selector, &[contract_address.into_()]),
        &[proxy_address.into_()],
    );
}

#[test]
fn cheat_caller_address_library_call() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("CheatCallerAddressChecker", &contracts_data);
    let lib_call_address = test_env.deploy("CheatCallerAddressCheckerLibCall", &[]);

    test_env.start_cheat_caller_address(lib_call_address, 123);

    let lib_call_selector = "get_caller_address_with_lib_call";
    assert_success(
        test_env.call_contract(&lib_call_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt::from(123)],
    );

    test_env.stop_cheat_caller_address(lib_call_address);

    assert_success(
        test_env.call_contract(&lib_call_address, lib_call_selector, &[class_hash.into_()]),
        &[TryFromHexStr::try_from_hex_str(TEST_ADDRESS).unwrap()],
    );
}

#[test]
fn cheat_caller_address_all() {
    let mut test_env = TestEnvironment::new();

    let cheat_caller_address_checker =
        test_env.declare("CheatCallerAddressChecker", &get_contracts());
    let contract_address = test_env.deploy_wrapper(&cheat_caller_address_checker, &[]);

    test_env
        .cheatnet_state
        .start_cheat_caller_address_global(123_u8.into());

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt::from(123)],
    );

    test_env.cheatnet_state.stop_cheat_caller_address_global();

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[TryFromHexStr::try_from_hex_str(TEST_ADDRESS).unwrap()],
    );

    let contract_address = test_env.deploy_wrapper(&cheat_caller_address_checker, &[]);

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[TryFromHexStr::try_from_hex_str(TEST_ADDRESS).unwrap()],
    );
}

#[test]
fn cheat_caller_address_multiple() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("CheatCallerAddressChecker", &contracts_data);

    let contract_address1 = test_env.deploy_wrapper(&class_hash, &[]);
    let contract_address2 = test_env.deploy_wrapper(&class_hash, &[]);

    test_env.start_cheat_caller_address(contract_address1, 123);
    test_env.start_cheat_caller_address(contract_address2, 123);

    assert_success(
        test_env.call_contract(&contract_address1, "get_caller_address", &[]),
        &[Felt::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address2, "get_caller_address", &[]),
        &[Felt::from(123)],
    );

    test_env
        .cheatnet_state
        .stop_cheat_caller_address(contract_address1);
    test_env
        .cheatnet_state
        .stop_cheat_caller_address(contract_address2);

    assert_success(
        test_env.call_contract(&contract_address1, "get_caller_address", &[]),
        &[TryFromHexStr::try_from_hex_str(TEST_ADDRESS).unwrap()],
    );
    assert_success(
        test_env.call_contract(&contract_address2, "get_caller_address", &[]),
        &[TryFromHexStr::try_from_hex_str(TEST_ADDRESS).unwrap()],
    );
}

#[test]
fn cheat_caller_address_all_then_one() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatCallerAddressChecker", &[]);

    test_env
        .cheatnet_state
        .start_cheat_caller_address_global(111_u8.into());
    test_env.start_cheat_caller_address(contract_address, 222);

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt::from(222)],
    );
}

#[test]
fn cheat_caller_address_one_then_all() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatCallerAddressChecker", &[]);

    test_env.start_cheat_caller_address(contract_address, 111);
    test_env
        .cheatnet_state
        .start_cheat_caller_address_global(222_u8.into());

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt::from(222)],
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cheat_caller_address_cairo0_callback() {
    let temp_dir = TempDir::new().unwrap();
    let cached_state = create_fork_cached_state_at(53_631, temp_dir.path().to_str().unwrap());
    let mut test_env = TestEnvironment::new();

    test_env.cached_state = cached_state;

    let contract_address = test_env.deploy("Cairo1Contract_v1", &[]);

    test_env.start_cheat_caller_address(contract_address, 123);

    let expected_caller_address = Felt::from(123);

    assert_success(
        test_env.call_contract(
            &contract_address,
            "start",
            &[
                // cairo 0 callback contract address
                Felt::try_from_hex_str(
                    "0x18783f6c124c3acc504f300cb6b3a33def439681744d027be8d7fd5d3551565",
                )
                .unwrap(),
                expected_caller_address,
            ],
        ),
        &[],
    );

    let events = test_env.cheatnet_state.get_events(0);

    // make sure end() was called by cairo0 contract
    assert_eq!(
        events[0],
        Event {
            from: contract_address,
            keys: vec![starknet_keccak("End".as_ref()).into()],
            data: vec![expected_caller_address]
        },
        "Wrong event"
    );
}

#[test]
fn cheat_caller_address_simple_with_span() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatCallerAddressChecker", &[]);

    test_env.cheat_caller_address(contract_address, 123, CheatSpan::TargetCalls(2));

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[TryFromHexStr::try_from_hex_str(TEST_ADDRESS).unwrap()],
    );
}

#[test]
fn cheat_caller_address_proxy_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("CheatCallerAddressCheckerProxy", &contracts_data);

    let contract_address_1 = test_env.deploy_wrapper(&class_hash, &[]);
    let contract_address_2 = test_env.deploy_wrapper(&class_hash, &[]);

    test_env.cheat_caller_address(contract_address_1, 123, CheatSpan::TargetCalls(1));

    let output = test_env.call_contract(
        &contract_address_1,
        "call_proxy",
        &[contract_address_2.into_()],
    );
    assert_success(output, &[123.into(), contract_address_2.into_()]);
}

#[test]
fn cheat_caller_address_override_span() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatCallerAddressChecker", &[]);

    test_env.cheat_caller_address(contract_address, 123, CheatSpan::TargetCalls(2));

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt::from(123)],
    );

    test_env.cheat_caller_address(contract_address, 321, CheatSpan::Indefinite);

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt::from(321)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt::from(321)],
    );

    test_env.stop_cheat_caller_address(contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[TryFromHexStr::try_from_hex_str(TEST_ADDRESS).unwrap()],
    );
}

#[test]
fn cheat_caller_address_constructor_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("ConstructorCheatCallerAddressChecker", &contracts_data);
    let precalculated_address = test_env.precalculate_address(&class_hash, &[]);

    test_env.cheat_caller_address(precalculated_address, 123, CheatSpan::TargetCalls(3));

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);
    assert_eq!(precalculated_address, contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_stored_caller_address", &[]),
        &[Felt::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[TryFromHexStr::try_from_hex_str(TEST_ADDRESS).unwrap()],
    );
}

#[test]
fn cheat_caller_address_library_call_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("CheatCallerAddressChecker", &contracts_data);
    let contract_address = test_env.deploy("CheatCallerAddressCheckerLibCall", &[]);

    test_env.cheat_caller_address(contract_address, 123, CheatSpan::TargetCalls(1));

    let lib_call_selector = "get_caller_address_with_lib_call";

    assert_success(
        test_env.call_contract(&contract_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, lib_call_selector, &[class_hash.into_()]),
        &[TryFromHexStr::try_from_hex_str(TEST_ADDRESS).unwrap()],
    );
}
