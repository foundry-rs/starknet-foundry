use crate::common::cache::{purge_cache, read_cache};
use crate::common::state::{
    create_cheatnet_state, create_fork_cached_state, create_fork_cached_state_at,
};
use crate::common::{call_contract, deploy_contract, felt_selector_from_name};
use crate::{assert_error, assert_success};
use blockifier::state::cached_state::{CachedState, GlobalContractCache};
use cairo_felt::Felt252;
use cairo_vm::vm::errors::hint_errors::HintError;
use cheatnet::constants::build_testing_state;
use cheatnet::forking::state::ForkStateReader;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::deploy::deploy;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::CheatcodeError;
use cheatnet::state::{BlockInfoReader, BlockifierState, CheatnetState, ExtendedStateReader};
use conversions::{IntoConv, TryIntoConv};
use num_bigint::BigUint;
use num_traits::Num;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use runtime::EnhancedHintError;
use serde_json::Value;
use starknet_api::block::BlockNumber;
use starknet_api::core::ContractAddress;
use std::path::PathBuf;
use std::str::FromStr;
use tempfile::TempDir;
use url::Url;

#[test]
fn fork_simple() {
    let cache_dir = TempDir::new().unwrap();
    let mut cached_fork_state = create_fork_cached_state(cache_dir.path().to_str().unwrap());
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_fork_state);

    let contract_address = Felt252::from(
        BigUint::from_str(
            "3216637956526895219277698311134811322769343974163380838558193911733621219342",
        )
        .unwrap(),
    )
    .into_();

    let selector = felt_selector_from_name("get_balance");
    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();
    assert_success!(output, vec![Felt252::from(2)]);

    let selector = felt_selector_from_name("increase_balance");
    call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[Felt252::from(100)],
    )
    .unwrap();

    let selector = felt_selector_from_name("get_balance");
    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();
    assert_success!(output, vec![Felt252::from(102)]);
}

#[test]
fn try_calling_nonexistent_contract() {
    let cache_dir = TempDir::new().unwrap();
    let mut cached_fork_state = create_fork_cached_state(cache_dir.path().to_str().unwrap());
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_fork_state);

    let contract_address = ContractAddress::from(1_u8);
    let selector = felt_selector_from_name("get_balance");

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();
    assert_error!(
        output,
        "Contract not deployed at address: 0x0000000000000000000000000000000000000000000000000000000000000001"
    );
}

#[test]
fn try_deploying_undeclared_class() {
    let cache_dir = TempDir::new().unwrap();
    let mut cached_fork_state = create_fork_cached_state(cache_dir.path().to_str().unwrap());
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_fork_state);

    let class_hash = "1".to_owned().try_into_().unwrap();

    assert!(
        match deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[]) {
            Err(CheatcodeError::Unrecoverable(EnhancedHintError::Hint(HintError::CustomHint(
                msg,
            )))) => msg.as_ref().contains(class_hash.to_string().as_str()),
            _ => false,
        }
    );
}

#[test]
fn test_forking_at_block_number() {
    let node_url: Url = "http://188.34.188.184:9545/rpc/v0_6".parse().unwrap();
    let cache_dir = TempDir::new().unwrap();

    {
        let mut cheatnet_state = CheatnetState::default();
        let mut cached_state_before_delopy = CachedState::new(
            ExtendedStateReader {
                dict_state_reader: build_testing_state(),
                fork_state_reader: Some(ForkStateReader::new(
                    node_url.clone(),
                    BlockNumber(309_780),
                    cache_dir.path().to_str().unwrap(),
                )),
            },
            GlobalContractCache::default(),
        );
        let mut state_before_deploy = BlockifierState::from(&mut cached_state_before_delopy);

        let cached_state_after_deploy = &mut CachedState::new(
            ExtendedStateReader {
                dict_state_reader: build_testing_state(),
                fork_state_reader: Some(ForkStateReader::new(
                    node_url,
                    BlockNumber(309_781),
                    cache_dir.path().to_str().unwrap(),
                )),
            },
            GlobalContractCache::default(),
        );
        let mut state_after_deploy = BlockifierState::from(cached_state_after_deploy);

        let contract_address = Felt252::from(
            BigUint::from_str(
                "3216637956526895219277698311134811322769343974163380838558193911733621219342",
            )
            .unwrap(),
        )
        .into_();

        let selector = felt_selector_from_name("get_balance");
        let output = call_contract(
            &mut state_before_deploy,
            &mut cheatnet_state,
            &contract_address,
            &selector,
            &[],
        )
        .unwrap();
        assert_error!(
            output,
            "Contract not deployed at address: 0x071c8d74edc89330f314f3b1109059d68ebfa68874aa91e9c425a6378ffde00e"
        );

        let selector = felt_selector_from_name("get_balance");
        let output = call_contract(
            &mut state_after_deploy,
            &mut cheatnet_state,
            &contract_address,
            &selector,
            &[],
        )
        .unwrap();
        assert_success!(output, vec![Felt252::from(0)]);
    }
    purge_cache(cache_dir.path().to_str().unwrap());
}

#[test]
fn call_forked_contract_from_other_contract() {
    let cache_dir = TempDir::new().unwrap();
    let mut cached_fork_state = create_fork_cached_state(cache_dir.path().to_str().unwrap());
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_fork_state);

    let forked_contract_address = Felt252::from(
        BigUint::from_str(
            "3216637956526895219277698311134811322769343974163380838558193911733621219342",
        )
        .unwrap(),
    );

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "ForkingChecker",
        &[Felt252::from(1)],
    );

    let selector = felt_selector_from_name("get_balance_call_contract");
    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[forked_contract_address],
    )
    .unwrap();
    assert_success!(output, vec![Felt252::from(2)]);
}

#[test]
fn library_call_on_forked_class_hash() {
    let cache_dir = TempDir::new().unwrap();
    let mut cached_fork_state = create_fork_cached_state(cache_dir.path().to_str().unwrap());
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_fork_state);

    let forked_class_hash = Felt252::from(
        BigUint::from_str(
            "2721209982346623666255046859539202086457905975723689966720503254490557413774",
        )
        .unwrap(),
    );

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "ForkingChecker",
        &[Felt252::from(1)],
    );

    let selector = felt_selector_from_name("get_balance_library_call");
    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[forked_class_hash.clone()],
    )
    .unwrap();
    assert_success!(output, vec![Felt252::from(0)]);

    call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &felt_selector_from_name("set_balance"),
        &[Felt252::from(100)],
    )
    .unwrap();

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[forked_class_hash],
    )
    .unwrap();
    assert_success!(output, vec![Felt252::from(100)]);
}

#[test]
fn call_forked_contract_from_constructor() {
    let cache_dir = TempDir::new().unwrap();
    let mut cached_fork_state = create_fork_cached_state(cache_dir.path().to_str().unwrap());
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_fork_state);

    let forked_class_hash = Felt252::from(
        BigUint::from_str(
            "2721209982346623666255046859539202086457905975723689966720503254490557413774",
        )
        .unwrap(),
    );

    let forked_contract_address = Felt252::from(
        BigUint::from_str(
            "3216637956526895219277698311134811322769343974163380838558193911733621219342",
        )
        .unwrap(),
    );

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "ForkingChecker",
        &[Felt252::from(0), forked_contract_address],
    );

    let selector = felt_selector_from_name("get_balance_library_call");
    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[forked_class_hash],
    )
    .unwrap();
    assert_success!(output, vec![Felt252::from(2)]);
}

#[test]
fn call_forked_contract_get_block_info_via_proxy() {
    let cache_dir = TempDir::new().unwrap();
    let mut cached_fork_state =
        create_fork_cached_state_at(BlockNumber(315_887), cache_dir.path().to_str().unwrap());
    let block_info = cached_fork_state.state.get_block_info().unwrap();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_fork_state);
    cheatnet_state.block_info = block_info;

    let forked_contract_address = Felt252::from(
        BigUint::from_str(
            "2142482702760034245482243841749569811658592971915399561448302710970247869206",
        )
        .unwrap(),
    );

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "BlockInfoCheckerProxy",
        &[],
    );

    let selector = felt_selector_from_name("read_block_number");
    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[forked_contract_address.clone()],
    )
    .unwrap();
    assert_success!(output, vec![Felt252::from(315_887)]);

    let selector = felt_selector_from_name("read_block_timestamp");
    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[forked_contract_address.clone()],
    )
    .unwrap();
    assert_success!(output, vec![Felt252::from(1_697_630_072)]);

    let selector = felt_selector_from_name("read_sequencer_address");
    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[forked_contract_address],
    )
    .unwrap();
    assert_success!(
        output,
        vec![Felt252::from(
            BigUint::from_str_radix(
                &"0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8"[2..],
                16
            )
            .unwrap()
        )]
    );
}

#[test]
fn call_forked_contract_get_block_info_via_libcall() {
    let cache_dir = TempDir::new().unwrap();
    let mut cached_fork_state =
        create_fork_cached_state_at(BlockNumber(315_887), cache_dir.path().to_str().unwrap());
    let block_info = cached_fork_state.state.get_block_info().unwrap();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_fork_state);
    cheatnet_state.block_info = block_info;

    let forked_class_hash = Felt252::from(
        BigUint::from_str_radix(
            &"0x00623d04363a9502cd0706dfc717574dd3c596f162c3867456002f25c706cd14"[2..],
            16,
        )
        .unwrap(),
    );

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "BlockInfoCheckerLibCall",
        &[],
    );

    let selector = felt_selector_from_name("read_block_number_with_lib_call");
    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[forked_class_hash.clone()],
    )
    .unwrap();
    assert_success!(output, vec![Felt252::from(315_887)]);

    let selector = felt_selector_from_name("read_block_timestamp_with_lib_call");
    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[forked_class_hash.clone()],
    )
    .unwrap();
    assert_success!(output, vec![Felt252::from(1_697_630_072)]);

    let selector = felt_selector_from_name("read_sequencer_address_with_lib_call");
    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[forked_class_hash],
    )
    .unwrap();
    assert_success!(
        output,
        vec![Felt252::from(
            BigUint::from_str_radix(
                &"0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8"[2..],
                16
            )
            .unwrap()
        )]
    );
}

#[test]
fn using_specified_block_nb_is_cached() {
    let cache_dir = TempDir::new().unwrap();
    let run_test = || {
        let mut cached_state =
            create_fork_cached_state_at(BlockNumber(312_646), cache_dir.path().to_str().unwrap());
        let _ = cached_state.state.get_block_info().unwrap();

        let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);
        let contract_address = Felt252::from(
            BigUint::from_str(
                "3216637956526895219277698311134811322769343974163380838558193911733621219342",
            )
            .unwrap(),
        )
        .into_();

        let selector = felt_selector_from_name("get_balance");
        let output = call_contract(
            &mut blockifier_state,
            &mut cheatnet_state,
            &contract_address,
            &selector,
            &[],
        )
        .unwrap();
        assert_success!(output, vec![Felt252::from(2)]);
    };

    let assert_cache = || {
        // Assertions
        let cache = read_cache(
            cache_dir
                .path()
                .join(PathBuf::from_str("*312646.json").unwrap())
                .to_str()
                .unwrap(),
        );
        assert_eq!(
            cache["storage_at"].as_object().unwrap()
                ["3216637956526895219277698311134811322769343974163380838558193911733621219342"]
                .as_object()
                .unwrap()
                ["916907772491729262376534102982219947830828984996257231353398618781993312401"],
            "2"
        );
        assert_eq!(
            cache["class_hash_at"].as_object().unwrap()
                ["3216637956526895219277698311134811322769343974163380838558193911733621219342"],
            "2721209982346623666255046859539202086457905975723689966720503254490557413774"
        );

        match cache["compiled_contract_class"].as_object().unwrap()
            ["2721209982346623666255046859539202086457905975723689966720503254490557413774"]
        {
            Value::String(_) => {}
            _ => panic!("The compiled_contract_class entry is not as string"),
        };

        assert_eq!(
            cache["block_info"].as_object().unwrap()["block_number"]
                .as_u64()
                .unwrap(),
            312_646
        );
        assert_eq!(
            cache["block_info"].as_object().unwrap()["timestamp"]
                .as_u64()
                .unwrap(),
            1_695_291_683
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
        let mut cached_state = create_fork_cached_state_at(BlockNumber(312_767), cache_dir);
        let _ = cached_state.state.get_block_info().unwrap();

        let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);
        let contract_address = Felt252::from(BigUint::from_str(contract_address).unwrap()).into_();

        let selector = felt_selector_from_name("get_balance");
        let output = call_contract(
            &mut blockifier_state,
            &mut cheatnet_state,
            &contract_address,
            &selector,
            &[],
        )
        .unwrap();
        assert_success!(output, vec![Felt252::from(balance)]);
    }

    let cache_dir = TempDir::new().unwrap();
    let contract_1_address =
        "3216637956526895219277698311134811322769343974163380838558193911733621219342";
    let contract_2_address =
        "3221247681918684045050855759557788124640099286968827281606334752803016107426";

    let assert_cache = || {
        // Assertions
        let cache = read_cache(
            cache_dir
                .path()
                .join(PathBuf::from_str("*312767.json").unwrap())
                .to_str()
                .unwrap(),
        );

        let contract_1_class_hash =
            "2721209982346623666255046859539202086457905975723689966720503254490557413774";
        let contract_2_class_hash =
            "1174898468974458236078282214054469608534299071801223106624371487328427954435";

        let balance_storage_address =
            "916907772491729262376534102982219947830828984996257231353398618781993312401";
        assert_eq!(
            cache["storage_at"].as_object().unwrap()[contract_1_address]
                .as_object()
                .unwrap()[balance_storage_address],
            "2"
        );
        assert_eq!(
            cache["storage_at"].as_object().unwrap()[contract_2_address]
                .as_object()
                .unwrap()[balance_storage_address],
            "0"
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
            Value::String(_) => {}
            _ => panic!("The compiled_contract_class entry is not as string"),
        }
        match cache["compiled_contract_class"].as_object().unwrap()[contract_2_class_hash] {
            Value::String(_) => {}
            _ => panic!("The compiled_contract_class entry is not as string"),
        }

        assert_eq!(
            cache["block_info"].as_object().unwrap()["block_number"]
                .as_u64()
                .unwrap(),
            312_767
        );
        assert_eq!(
            cache["block_info"].as_object().unwrap()["timestamp"]
                .as_u64()
                .unwrap(),
            1_695_378_726
        );
        assert_eq!(
            cache["block_info"].as_object().unwrap()["sequencer_address"],
            "0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8"
        );
    };
    let cache_dir_str = cache_dir.path().to_str().unwrap();

    run_test(cache_dir_str, contract_1_address, 2);
    run_test(cache_dir_str, contract_2_address, 0);
    assert_cache();

    purge_cache(cache_dir.path().to_str().unwrap());

    // Parallel execution
    vec![
        (cache_dir_str, contract_1_address, 2),
        (cache_dir_str, contract_2_address, 0),
    ]
    .par_iter()
    .for_each(|param_tpl| run_test(param_tpl.0, param_tpl.1, param_tpl.2));

    assert_cache();
}

#[test]
fn test_cached_block_info_merging() {
    fn run_test(cache_dir: &str, contract_address: &str, balance: u64, call_get_block_info: bool) {
        let mut cached_state = create_fork_cached_state_at(BlockNumber(312_767), cache_dir);
        if call_get_block_info {
            let _ = cached_state.state.get_block_info().unwrap();
        }
        let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);
        let contract_address = Felt252::from(BigUint::from_str(contract_address).unwrap()).into_();

        let selector = felt_selector_from_name("get_balance");
        let output = call_contract(
            &mut blockifier_state,
            &mut cheatnet_state,
            &contract_address,
            &selector,
            &[],
        )
        .unwrap();
        assert_success!(output, vec![Felt252::from(balance)]);
    }

    let cache_dir = TempDir::new().unwrap();
    let contract_1_address =
        "3216637956526895219277698311134811322769343974163380838558193911733621219342";

    let assert_cached_block_info = |is_block_info_cached: bool| {
        // Assertions
        let cache = read_cache(
            cache_dir
                .path()
                .join(PathBuf::from_str("*312767.json").unwrap())
                .to_str()
                .unwrap(),
        );
        if is_block_info_cached {
            assert_eq!(
                cache["block_info"].as_object().unwrap()["block_number"]
                    .as_u64()
                    .unwrap(),
                312_767
            );
            assert_eq!(
                cache["block_info"].as_object().unwrap()["timestamp"]
                    .as_u64()
                    .unwrap(),
                1_695_378_726
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

    run_test(cache_dir_str, contract_1_address, 2, false);
    assert_cached_block_info(false);
    run_test(cache_dir_str, contract_1_address, 2, true);
    assert_cached_block_info(true);
    run_test(cache_dir_str, contract_1_address, 2, false);
    assert_cached_block_info(true);
}

#[test]
fn test_calling_nonexistent_url() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent_url = "http://188.34.188.184:9546".parse().unwrap();
    let mut cached_fork_state = CachedState::new(
        ExtendedStateReader {
            dict_state_reader: build_testing_state(),
            fork_state_reader: Some(ForkStateReader::new(
                nonexistent_url,
                BlockNumber(1),
                temp_dir.path().to_str().unwrap(),
            )),
        },
        GlobalContractCache::default(),
    );

    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_fork_state);

    let contract_address = Felt252::from(
        BigUint::from_str(
            "3216637956526895219277698311134811322769343974163380838558193911733621219342",
        )
        .unwrap(),
    )
    .into_();

    let selector = felt_selector_from_name("get_balance");
    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    assert_error!(
        output,
        "Unable to reach the node. Check your internet connection and node url"
    );
}
