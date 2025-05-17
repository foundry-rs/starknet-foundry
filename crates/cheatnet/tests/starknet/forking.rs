use crate::common::assertions::{assert_error, assert_panic, assert_success};
use crate::common::cache::{purge_cache, read_cache};
use crate::common::state::{create_fork_cached_state, create_fork_cached_state_at};
use crate::common::{call_contract, deploy_contract, deploy_wrapper};
use blockifier::state::cached_state::CachedState;
use cairo_vm::vm::errors::hint_errors::HintError;
use camino::Utf8Path;
use cheatnet::constants::build_testing_state;
use cheatnet::forking::cache::cache_version;
use cheatnet::forking::state::ForkStateReader;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::CheatcodeError;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::storage::selector_from_name;
use cheatnet::state::{BlockInfoReader, CheatnetState, ExtendedStateReader};
use conversions::IntoConv;
use conversions::byte_array::ByteArray;
use conversions::string::TryFromHexStr;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use runtime::EnhancedHintError;
use serde_json::Value;
use starknet_api::block::BlockNumber;
use starknet_api::core::ContractAddress;
use starknet_types_core::felt::Felt;
use tempfile::TempDir;

#[test]
fn fork_simple() {
    let cache_dir = TempDir::new().unwrap();
    let mut cached_fork_state = create_fork_cached_state(cache_dir.path().to_str().unwrap());
    let mut cheatnet_state = CheatnetState::default();

    let contract_address = ContractAddress::try_from_hex_str(
        "0x202de98471a4fae6bcbabb96cab00437d381abc58b02509043778074d6781e9",
    )
    .unwrap();

    let selector = selector_from_name("get_balance");
    let output = call_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );
    assert_success(output, &[Felt::from(0)]);

    let selector = selector_from_name("increase_balance");
    call_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[Felt::from(100)],
    );

    let selector = selector_from_name("get_balance");
    let output = call_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );
    assert_success(output, &[Felt::from(100)]);
}

#[test]
fn try_calling_nonexistent_contract() {
    let cache_dir = TempDir::new().unwrap();
    let mut cached_fork_state = create_fork_cached_state(cache_dir.path().to_str().unwrap());
    let mut cheatnet_state = CheatnetState::default();

    let contract_address = ContractAddress::from(1_u8);
    let selector = selector_from_name("get_balance");

    let output = call_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );

    let msg = "Contract not deployed at address: 0x1";
    let panic_data_felts: Vec<Felt> = ByteArray::from(msg).serialize_with_magic();
    assert_panic(output, &panic_data_felts);
}

#[test]
fn try_deploying_undeclared_class() {
    let cache_dir = TempDir::new().unwrap();
    let mut cached_fork_state = create_fork_cached_state(cache_dir.path().to_str().unwrap());
    let mut cheatnet_state = CheatnetState::default();

    let class_hash = Felt::ONE.into_();

    assert!(match deploy_wrapper(
        &mut cached_fork_state,
        &mut cheatnet_state,
        &class_hash,
        &[]
    ) {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Hint(HintError::CustomHint(msg)))) =>
            msg.as_ref().contains(class_hash.to_string().as_str()),
        _ => false,
    });
}

#[test]
fn test_forking_at_block_number() {
    let cache_dir = TempDir::new().unwrap();

    {
        let mut cheatnet_state = CheatnetState::default();
        let mut cached_state_before_delopy =
            create_fork_cached_state_at(50_000, cache_dir.path().to_str().unwrap());

        let mut cached_state_after_deploy =
            create_fork_cached_state_at(53_681, cache_dir.path().to_str().unwrap());

        let contract_address = ContractAddress::try_from_hex_str(
            "0x202de98471a4fae6bcbabb96cab00437d381abc58b02509043778074d6781e9",
        )
        .unwrap();

        let selector = selector_from_name("get_balance");
        let output = call_contract(
            &mut cached_state_before_delopy,
            &mut cheatnet_state,
            &contract_address,
            selector,
            &[],
        );

        let msg = "Contract not deployed at address: 0x202de98471a4fae6bcbabb96cab00437d381abc58b02509043778074d6781e9";
        let panic_data_felts: Vec<Felt> = ByteArray::from(msg).serialize_with_magic();
        assert_panic(output, &panic_data_felts);

        let selector = selector_from_name("get_balance");
        let output = call_contract(
            &mut cached_state_after_deploy,
            &mut cheatnet_state,
            &contract_address,
            selector,
            &[],
        );

        assert_success(output, &[Felt::from(0)]);
    }

    purge_cache(cache_dir.path().to_str().unwrap());
}

#[test]
fn call_forked_contract_from_other_contract() {
    let cache_dir = TempDir::new().unwrap();
    let mut cached_fork_state = create_fork_cached_state(cache_dir.path().to_str().unwrap());
    let mut cheatnet_state = CheatnetState::default();

    let forked_contract_address =
        Felt::try_from_hex_str("0x202de98471a4fae6bcbabb96cab00437d381abc58b02509043778074d6781e9")
            .unwrap();

    let contract_address = deploy_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        "ForkingChecker",
        &[Felt::from(1)],
    );

    let selector = selector_from_name("get_balance_call_contract");
    let output = call_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[forked_contract_address],
    );
    assert_success(output, &[Felt::from(0)]);
}

#[test]
fn library_call_on_forked_class_hash() {
    let cache_dir = TempDir::new().unwrap();
    let mut cached_fork_state = create_fork_cached_state(cache_dir.path().to_str().unwrap());
    let mut cheatnet_state = CheatnetState::default();

    let forked_class_hash = Felt::try_from_hex_str(
        "0x06a7eb29ee38b0a0b198e39ed6ad458d2e460264b463351a0acfc05822d61550",
    )
    .unwrap();

    let contract_address = deploy_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        "ForkingChecker",
        &[Felt::from(1)],
    );

    let selector = selector_from_name("get_balance_library_call");
    let output = call_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[forked_class_hash],
    );
    assert_success(output, &[Felt::from(0)]);

    call_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        &contract_address,
        selector_from_name("set_balance"),
        &[Felt::from(100)],
    );

    let output = call_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[forked_class_hash],
    );
    assert_success(output, &[Felt::from(100)]);
}

#[test]
fn call_forked_contract_from_constructor() {
    let cache_dir = TempDir::new().unwrap();
    let mut cached_fork_state = create_fork_cached_state(cache_dir.path().to_str().unwrap());
    let mut cheatnet_state = CheatnetState::default();

    let forked_class_hash = Felt::try_from_hex_str(
        "0x06a7eb29ee38b0a0b198e39ed6ad458d2e460264b463351a0acfc05822d61550",
    )
    .unwrap();

    let forked_contract_address =
        Felt::try_from_hex_str("0x202de98471a4fae6bcbabb96cab00437d381abc58b02509043778074d6781e9")
            .unwrap();

    let contract_address = deploy_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        "ForkingChecker",
        &[Felt::from(0), forked_contract_address],
    );

    let selector = selector_from_name("get_balance_library_call");
    let output = call_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[forked_class_hash],
    );
    assert_success(output, &[Felt::from(0)]);
}

#[test]
fn call_forked_contract_get_block_info_via_proxy() {
    let cache_dir = TempDir::new().unwrap();
    let mut cached_fork_state =
        create_fork_cached_state_at(53_655, cache_dir.path().to_str().unwrap());
    let block_info = cached_fork_state.state.get_block_info().unwrap();
    let mut cheatnet_state = CheatnetState {
        block_info,
        ..Default::default()
    };

    let forked_contract_address =
        Felt::try_from_hex_str("0x3d80c579ad7d83ff46634abe8f91f9d2080c5c076d4f0f59dd810f9b3f01164")
            .unwrap();

    let contract_address = deploy_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        "BlockInfoCheckerProxy",
        &[],
    );

    let selector = selector_from_name("read_block_number");
    let output = call_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[forked_contract_address],
    );
    assert_success(output, &[Felt::from(53_655)]);

    let selector = selector_from_name("read_block_timestamp");
    let output = call_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[forked_contract_address],
    );
    assert_success(output, &[Felt::from(1_711_548_115)]);

    let selector = selector_from_name("read_sequencer_address");
    let output = call_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[forked_contract_address],
    );
    assert_success(
        output,
        &[Felt::try_from_hex_str(
            "0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8",
        )
        .unwrap()],
    );
}

#[test]
fn call_forked_contract_get_block_info_via_libcall() {
    let cache_dir = TempDir::new().unwrap();
    let mut cached_fork_state =
        create_fork_cached_state_at(53_669, cache_dir.path().to_str().unwrap());
    let block_info = cached_fork_state.state.get_block_info().unwrap();
    let mut cheatnet_state = CheatnetState {
        block_info,
        ..Default::default()
    };

    let forked_class_hash = Felt::try_from_hex_str(
        "0x04947e141416a51b57a59bc8786b5c0e02751d33e46383fa9cebbf9cf6f30844",
    )
    .unwrap();

    let contract_address = deploy_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        "BlockInfoCheckerLibCall",
        &[],
    );

    let selector = selector_from_name("read_block_number_with_lib_call");
    let output = call_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[forked_class_hash],
    );
    assert_success(output, &[Felt::from(53_669)]);

    let selector = selector_from_name("read_block_timestamp_with_lib_call");
    let output = call_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[forked_class_hash],
    );
    assert_success(output, &[Felt::from(1_711_551_518)]);

    let selector = selector_from_name("read_sequencer_address_with_lib_call");
    let output = call_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[forked_class_hash],
    );
    assert_success(
        output,
        &[Felt::try_from_hex_str(
            "0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8",
        )
        .unwrap()],
    );
}

#[test]
fn using_specified_block_nb_is_cached() {
    let cache_dir = TempDir::new().unwrap();
    let run_test = || {
        let mut cached_state =
            create_fork_cached_state_at(53_669, cache_dir.path().to_str().unwrap());
        let _ = cached_state.state.get_block_info().unwrap();

        let mut cheatnet_state = CheatnetState::default();

        let contract_address = ContractAddress::try_from_hex_str(
            "0x202de98471a4fae6bcbabb96cab00437d381abc58b02509043778074d6781e9",
        )
        .unwrap();

        let selector = selector_from_name("get_balance");
        let output = call_contract(
            &mut cached_state,
            &mut cheatnet_state,
            &contract_address,
            selector,
            &[],
        );

        assert_success(output, &[Felt::from(0)]);
    };

    let assert_cache = || {
        // Assertions
        let cache = read_cache(
            cache_dir
                .path()
                .join(format!("*v{}.json", cache_version()))
                .to_str()
                .unwrap(),
        );
        assert_eq!(
            cache["storage_at"].as_object().unwrap()
                ["0x202de98471a4fae6bcbabb96cab00437d381abc58b02509043778074d6781e9"]
                .as_object()
                .unwrap()["0x206f38f7e4f15e87567361213c28f235cccdaa1d7fd34c9db1dfe9489c6a091"],
            "0x0"
        );
        assert_eq!(
            cache["class_hash_at"].as_object().unwrap()["0x202de98471a4fae6bcbabb96cab00437d381abc58b02509043778074d6781e9"],
            "0x6a7eb29ee38b0a0b198e39ed6ad458d2e460264b463351a0acfc05822d61550"
        );

        match cache["compiled_contract_class"].as_object().unwrap()["0x6a7eb29ee38b0a0b198e39ed6ad458d2e460264b463351a0acfc05822d61550"]
        {
            Value::Object(_) => {}
            _ => panic!("The compiled_contract_class entry is not an object"),
        }

        assert_eq!(
            cache["block_info"].as_object().unwrap()["block_number"]
                .as_u64()
                .unwrap(),
            53_669
        );
        assert_eq!(
            cache["block_info"].as_object().unwrap()["block_timestamp"]
                .as_u64()
                .unwrap(),
            1_711_551_518
        );
        assert_eq!(
            cache["block_info"].as_object().unwrap()["sequencer_address"],
            "0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8"
        );
    };
    // 1st run - check whether cache is written
    run_test();
    assert_cache();
    // 2nd run - check whether cache still the same after, as after the 1st
    run_test();
    assert_cache();

    purge_cache(cache_dir.path().to_str().unwrap());
}

#[test]
fn test_cache_merging() {
    fn run_test(cache_dir: &str, contract_address: &str, balance: u64) {
        let mut cached_state = create_fork_cached_state_at(53_680, cache_dir);
        let _ = cached_state.state.get_block_info().unwrap();

        let mut cheatnet_state = CheatnetState::default();

        let contract_address = ContractAddress::try_from_hex_str(contract_address).unwrap();

        let selector = selector_from_name("get_balance");
        let output = call_contract(
            &mut cached_state,
            &mut cheatnet_state,
            &contract_address,
            selector,
            &[],
        );

        assert_success(output, &[Felt::from(balance)]);
    }

    let cache_dir = TempDir::new().unwrap();
    let contract_1_address = "0x202de98471a4fae6bcbabb96cab00437d381abc58b02509043778074d6781e9";
    let contract_2_address = "0x4e6e4924e5db5ffe394484860a8f60e5c292d1937fd80040b312aeea921be11";

    let assert_cache = || {
        // Assertions
        let cache = read_cache(
            cache_dir
                .path()
                .join(format!("*v{}.json", cache_version()))
                .to_str()
                .unwrap(),
        );

        let contract_1_class_hash =
            "0x6a7eb29ee38b0a0b198e39ed6ad458d2e460264b463351a0acfc05822d61550";
        let contract_2_class_hash =
            "0x2b08ef708af3e6263e02ce541a0099e7c30bac5a8d3d13e42c25c787fa4163";

        let balance_storage_address =
            "0x206f38f7e4f15e87567361213c28f235cccdaa1d7fd34c9db1dfe9489c6a091";
        assert_eq!(
            cache["storage_at"].as_object().unwrap()[contract_1_address]
                .as_object()
                .unwrap()[balance_storage_address],
            "0x0"
        );
        assert_eq!(
            cache["storage_at"].as_object().unwrap()[contract_2_address]
                .as_object()
                .unwrap()[balance_storage_address],
            "0x0"
        );

        assert_eq!(
            cache["class_hash_at"].as_object().unwrap()[contract_1_address],
            contract_1_class_hash
        );

        assert_eq!(
            cache["class_hash_at"].as_object().unwrap()[contract_2_address],
            contract_2_class_hash
        );

        match cache["compiled_contract_class"].as_object().unwrap()[contract_1_class_hash] {
            Value::Object(_) => {}
            _ => panic!("The compiled_contract_class entry is not an object"),
        }
        match cache["compiled_contract_class"].as_object().unwrap()[contract_2_class_hash] {
            Value::Object(_) => {}
            _ => panic!("The compiled_contract_class entry is not an object"),
        }

        assert_eq!(
            cache["block_info"].as_object().unwrap()["block_number"]
                .as_u64()
                .unwrap(),
            53_680
        );
        assert_eq!(
            cache["block_info"].as_object().unwrap()["block_timestamp"]
                .as_u64()
                .unwrap(),
            1_711_554_206
        );
        assert_eq!(
            cache["block_info"].as_object().unwrap()["sequencer_address"],
            "0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8"
        );
    };
    let cache_dir_str = cache_dir.path().to_str().unwrap();

    run_test(cache_dir_str, contract_1_address, 0);
    run_test(cache_dir_str, contract_2_address, 0);
    assert_cache();

    purge_cache(cache_dir.path().to_str().unwrap());

    // Parallel execution
    [
        (cache_dir_str, contract_1_address, 0),
        (cache_dir_str, contract_2_address, 0),
    ]
    .par_iter()
    .for_each(|param_tpl| run_test(param_tpl.0, param_tpl.1, param_tpl.2));

    assert_cache();
}

#[test]
fn test_cached_block_info_merging() {
    fn run_test(cache_dir: &str, balance: u64, call_get_block_info: bool) {
        let mut cached_state = create_fork_cached_state_at(53_680, cache_dir);
        if call_get_block_info {
            let _ = cached_state.state.get_block_info().unwrap();
        }
        let mut cheatnet_state = CheatnetState::default();

        let contract_address = ContractAddress::try_from_hex_str(
            "0x202de98471a4fae6bcbabb96cab00437d381abc58b02509043778074d6781e9",
        )
        .unwrap();

        let selector = selector_from_name("get_balance");
        let output = call_contract(
            &mut cached_state,
            &mut cheatnet_state,
            &contract_address,
            selector,
            &[],
        );

        assert_success(output, &[Felt::from(balance)]);
    }

    let cache_dir = TempDir::new().unwrap();

    let assert_cached_block_info = |is_block_info_cached: bool| {
        // Assertions
        let cache = read_cache(
            cache_dir
                .path()
                .join(format!("*v{}.json", cache_version()))
                .to_str()
                .unwrap(),
        );
        if is_block_info_cached {
            assert_eq!(
                cache["block_info"].as_object().unwrap()["block_number"]
                    .as_u64()
                    .unwrap(),
                53_680
            );
            assert_eq!(
                cache["block_info"].as_object().unwrap()["block_timestamp"]
                    .as_u64()
                    .unwrap(),
                1_711_554_206
            );
            assert_eq!(
                cache["block_info"].as_object().unwrap()["sequencer_address"],
                "0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8"
            );
        } else {
            assert_eq!(cache["block_info"].as_object(), None);
        }
    };
    let cache_dir_str = cache_dir.path().to_str().unwrap();

    run_test(cache_dir_str, 0, false);
    assert_cached_block_info(false);
    run_test(cache_dir_str, 0, true);
    assert_cached_block_info(true);
    run_test(cache_dir_str, 0, false);
    assert_cached_block_info(true);
}

#[test]
fn test_calling_nonexistent_url() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent_url = "http://nonexistent-node-address.com".parse().unwrap();
    let mut cached_fork_state = CachedState::new(ExtendedStateReader {
        dict_state_reader: build_testing_state(),
        fork_state_reader: Some(
            ForkStateReader::new(
                nonexistent_url,
                BlockNumber(1),
                Utf8Path::from_path(temp_dir.path()).unwrap(),
            )
            .unwrap(),
        ),
    });

    let mut cheatnet_state = CheatnetState::default();

    let contract_address = ContractAddress::try_from_hex_str(
        "0x202de98471a4fae6bcbabb96cab00437d381abc58b02509043778074d6781e9",
    )
    .unwrap();

    let selector = selector_from_name("get_balance");
    let output = call_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );

    assert_error(
        output,
        "Unable to reach the node. Check your internet connection and node url",
    );
}
