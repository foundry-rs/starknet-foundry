use crate::common::state::{create_cheatnet_fork_state, create_cheatnet_fork_state_at};
use crate::common::{deploy_contract, felt_selector_from_name};
use crate::{assert_error, assert_success};
use cairo_felt::Felt252;
use cairo_vm::vm::errors::hint_errors::HintError;
use camino::Utf8PathBuf;
use cheatnet::cheatcodes::{CheatcodeError, EnhancedHintError};
use cheatnet::constants::build_testing_state;
use cheatnet::forking::state::ForkStateReader;
use cheatnet::rpc::call_contract;
use cheatnet::state::ExtendedStateReader;
use cheatnet::CheatnetState;
use conversions::StarknetConversions;
use std::path::PathBuf;

use crate::common::cache::{purge_cache, read_cache};
use glob::glob;
use num_bigint::BigUint;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use serde_json::Value;
use starknet::core::types::{BlockId, BlockTag};
use starknet_api::core::ContractAddress;
use std::str::FromStr;
use tempfile::TempDir;

#[test]
fn fork_simple() {
    let mut state = create_cheatnet_fork_state();

    let contract_address = Felt252::from(
        BigUint::from_str(
            "3216637956526895219277698311134811322769343974163380838558193911733621219342",
        )
        .unwrap(),
    )
    .to_contract_address();

    let selector = felt_selector_from_name("get_balance");
    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();
    assert_success!(output, vec![Felt252::from(2)]);

    let selector = felt_selector_from_name("increase_balance");
    call_contract(
        &contract_address,
        &selector,
        &[Felt252::from(100)],
        &mut state,
    )
    .unwrap();

    let selector = felt_selector_from_name("get_balance");
    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();
    assert_success!(output, vec![Felt252::from(102)]);
}

#[test]
fn try_calling_nonexistent_contract() {
    let mut state = create_cheatnet_fork_state();

    let contract_address = ContractAddress::from(1_u8);
    let selector = felt_selector_from_name("get_balance");

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();
    assert_error!(
        output,
        "Contract not deployed at address: 0x0000000000000000000000000000000000000000000000000000000000000001"
    );
}

#[test]
fn try_deploying_undeclared_class() {
    let mut state = create_cheatnet_fork_state();

    let class_hash = "1".to_owned().to_class_hash();

    assert!(match state.deploy(&class_hash, &[]) {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Hint(HintError::CustomHint(msg)))) =>
            msg.as_ref().contains(class_hash.to_string().as_str()),
        _ => false,
    });
}

#[test]
fn test_forking_at_block_number() {
    let predeployed_contracts = Utf8PathBuf::from("predeployed-contracts");
    let node_url = "http://188.34.188.184:9545/rpc/v0.4";
    let cache_dir = TempDir::new().unwrap();

    {
        let mut state_before_deploy = CheatnetState::new(ExtendedStateReader {
            dict_state_reader: build_testing_state(&predeployed_contracts),
            fork_state_reader: Some(ForkStateReader::new(
                node_url,
                BlockId::Number(309_780),
                Some(cache_dir.path().to_str().unwrap()),
            )),
        });

        let mut state_after_deploy = CheatnetState::new(ExtendedStateReader {
            dict_state_reader: build_testing_state(&predeployed_contracts),
            fork_state_reader: Some(ForkStateReader::new(
                node_url,
                BlockId::Number(309_781),
                Some(cache_dir.path().to_str().unwrap()),
            )),
        });

        let contract_address = Felt252::from(
            BigUint::from_str(
                "3216637956526895219277698311134811322769343974163380838558193911733621219342",
            )
            .unwrap(),
        )
        .to_contract_address();

        let selector = felt_selector_from_name("get_balance");
        let output =
            call_contract(&contract_address, &selector, &[], &mut state_before_deploy).unwrap();
        assert_error!(
            output,
            "Contract not deployed at address: 0x071c8d74edc89330f314f3b1109059d68ebfa68874aa91e9c425a6378ffde00e"
        );

        let selector = felt_selector_from_name("get_balance");
        let output =
            call_contract(&contract_address, &selector, &[], &mut state_after_deploy).unwrap();
        assert_success!(output, vec![Felt252::from(0)]);
    }
    purge_cache(cache_dir.path().to_str().unwrap());
}

#[test]
fn call_forked_contract_from_other_contract() {
    let mut state = create_cheatnet_fork_state();

    let forked_contract_address = Felt252::from(
        BigUint::from_str(
            "3216637956526895219277698311134811322769343974163380838558193911733621219342",
        )
        .unwrap(),
    );

    let contract_address = deploy_contract(&mut state, "ForkingChecker", &[Felt252::from(1)]);

    let selector = felt_selector_from_name("get_balance_call_contract");
    let output = call_contract(
        &contract_address,
        &selector,
        &[forked_contract_address],
        &mut state,
    )
    .unwrap();
    assert_success!(output, vec![Felt252::from(2)]);
}

#[test]
fn library_call_on_forked_class_hash() {
    let mut state = create_cheatnet_fork_state();

    let forked_class_hash = Felt252::from(
        BigUint::from_str(
            "2721209982346623666255046859539202086457905975723689966720503254490557413774",
        )
        .unwrap(),
    );

    let contract_address = deploy_contract(&mut state, "ForkingChecker", &[Felt252::from(1)]);

    let selector = felt_selector_from_name("get_balance_library_call");
    let output = call_contract(
        &contract_address,
        &selector,
        &[forked_class_hash.clone()],
        &mut state,
    )
    .unwrap();
    assert_success!(output, vec![Felt252::from(0)]);

    call_contract(
        &contract_address,
        &felt_selector_from_name("set_balance"),
        &[Felt252::from(100)],
        &mut state,
    )
    .unwrap();

    let output = call_contract(
        &contract_address,
        &selector,
        &[forked_class_hash],
        &mut state,
    )
    .unwrap();
    assert_success!(output, vec![Felt252::from(100)]);
}

#[test]
fn call_forked_contract_from_constructor() {
    let mut state = create_cheatnet_fork_state();

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
        &mut state,
        "ForkingChecker",
        &[Felt252::from(0), forked_contract_address],
    );

    let selector = felt_selector_from_name("get_balance_library_call");
    let output = call_contract(
        &contract_address,
        &selector,
        &[forked_class_hash],
        &mut state,
    )
    .unwrap();
    assert_success!(output, vec![Felt252::from(2)]);
}

#[test]
fn using_specified_block_nb_is_cached() {
    let cache_dir = TempDir::new().unwrap();
    let run_test = || {
        let mut state = create_cheatnet_fork_state_at(
            BlockId::Number(312_646),
            cache_dir.path().to_str().unwrap(),
        );
        let contract_address = Felt252::from(
            BigUint::from_str(
                "3216637956526895219277698311134811322769343974163380838558193911733621219342",
            )
            .unwrap(),
        )
        .to_contract_address();

        let selector = felt_selector_from_name("get_balance");
        let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();
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
        }
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
fn using_block_tag_is_not_cached() {
    fn test_tag(tag: BlockTag) {
        let cache_dir = TempDir::new().unwrap();
        {
            let mut state = create_cheatnet_fork_state_at(
                BlockId::Tag(tag),
                cache_dir.path().to_str().unwrap(),
            );
            let contract_address = Felt252::from(
                BigUint::from_str(
                    "3216637956526895219277698311134811322769343974163380838558193911733621219342",
                )
                .unwrap(),
            )
            .to_contract_address();

            let selector = felt_selector_from_name("get_balance");
            let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();
            assert_success!(output, vec![Felt252::from(2)]);
        }

        let cache_files: Vec<PathBuf> = glob(
            cache_dir
                .path()
                .join(PathBuf::from_str("*latest.json").unwrap())
                .to_str()
                .unwrap(),
        )
        .unwrap()
        .filter_map(Result::ok)
        .collect();

        assert!(
            cache_files.is_empty(),
            "Cache file found for uncacheable tag"
        );
    }
    test_tag(BlockTag::Latest);
    test_tag(BlockTag::Pending);
}

#[test]
fn test_cache_merging() {
    fn run_test(cache_dir: &str, contract_address: &str, balance: u64) {
        let mut state = create_cheatnet_fork_state_at(BlockId::Number(312_767), cache_dir);
        let contract_address =
            Felt252::from(BigUint::from_str(contract_address).unwrap()).to_contract_address();

        let selector = felt_selector_from_name("get_balance");
        let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();
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
