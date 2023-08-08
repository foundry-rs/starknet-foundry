use crate::integration::common::corelib::{corelib_path, predeployed_contracts};
use crate::integration::common::runner::Contract;
use crate::{assert_passed, test_case};
use camino::Utf8PathBuf;
use forge::run;
use indoc::indoc;
use std::path::Path;

#[test]
fn start_mock_call_simple() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use cheatcodes::{ declare, PreparedContract, deploy, start_mock_call };

        #[starknet::interface]
        trait IMockChecker<TContractState> {
            fn get_thing(ref self: TContractState) -> felt252;
        }

        #[test]
        fn start_mock_call_simple() {
            let calldata = array![420];

            let class_hash = declare('MockChecker');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let contract_address = deploy(prepared).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mock_ret_data = array![421];
            start_mock_call(contract_address, 'get_thing', mock_ret_data);

            let thing = dispatcher.get_thing();

            assert(thing == 421, 'Incorrect thing');
        }

        #[test]
        fn start_mock_call_simple_mock_before_dispatcher_created() {
            let calldata = array![420];

            let class_hash = declare('MockChecker');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let contract_address = deploy(prepared).unwrap();

            let mock_ret_data = array![421];
            start_mock_call(contract_address, 'get_thing', mock_ret_data);

            let dispatcher = IMockCheckerDispatcher { contract_address };
            let thing = dispatcher.get_thing();

            assert(thing == 421, 'Incorrect thing');
        }
    "#
        ),
        Contract::from_code_path(
            "MockChecker".to_string(),
            Path::new("tests/data/contracts/mock_checker.cairo"),
        )
        .unwrap()
    );

    let result = run(
        &test.path().unwrap(),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();
    assert_passed!(result);
}

#[test]
fn stop_mock_call_simple() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use cheatcodes::{ declare, PreparedContract, deploy, start_mock_call, stop_mock_call };

        #[starknet::interface]
        trait IMockChecker<TContractState> {
            fn get_thing(ref self: TContractState) -> felt252;
        }

        #[test]
        fn stop_mock_call_simple() {
            let calldata = array![420];

            let class_hash = declare('MockChecker');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let contract_address = deploy(prepared).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mock_ret_data = array![421];

            start_mock_call(contract_address, 'get_thing', mock_ret_data);
            let thing = dispatcher.get_thing();
            assert(thing == 421, 'Incorrect thing');

            stop_mock_call(contract_address, 'get_thing');
            let thing = dispatcher.get_thing();
            assert(thing == 420, 'Incorrect thing');
        }

        #[test]
        fn stop_mock_call_when_mock_not_started() {
            let calldata = array![420];

            let class_hash = declare('MockChecker');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let contract_address = deploy(prepared).unwrap();

            stop_mock_call(contract_address, 'get_thing');

            let dispatcher = IMockCheckerDispatcher { contract_address };
            let thing = dispatcher.get_thing();

            assert(thing == 420, 'Incorrect thing');
        }
    "#
        ),
        Contract::from_code_path(
            "MockChecker".to_string(),
            Path::new("tests/data/contracts/mock_checker.cairo"),
        )
        .unwrap()
    );

    let result = run(
        &test.path().unwrap(),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();
    assert_passed!(result);
}

#[test]
fn mock_call_double() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use cheatcodes::{ declare, PreparedContract, deploy, start_mock_call, stop_mock_call };

        #[starknet::interface]
        trait IMockChecker<TContractState> {
            fn get_thing(ref self: TContractState) -> felt252;
        }

        #[test]
        fn mock_call_double_mocks() {
            let calldata = array![420];

            let class_hash = declare('MockChecker');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let contract_address = deploy(prepared).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mock_ret_data = array![421];
            start_mock_call(contract_address, 'get_thing', mock_ret_data);

            let  mock_ret_data = array![427];
            start_mock_call(contract_address, 'get_thing', mock_ret_data);

            let thing = dispatcher.get_thing();
            assert(thing == 427, 'Incorrect thing');

            stop_mock_call(contract_address, 'get_thing');
            let thing = dispatcher.get_thing();
            assert(thing == 420, 'Incorrect thing');
        }

        #[test]
        fn mock_call_double_calls() {
            let calldata = array![420];

            let class_hash = declare('MockChecker');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let contract_address = deploy(prepared).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mock_ret_data = array![421];
            start_mock_call(contract_address, 'get_thing', mock_ret_data);

            let thing1 = dispatcher.get_thing();
            let thing2 = dispatcher.get_thing();

            assert(thing1 == 421, 'Incorrect thing');
            assert(thing2 == 421, 'Incorrect thing');

            stop_mock_call(contract_address, 'get_thing');
            let thing = dispatcher.get_thing();
            assert(thing == 420, 'Incorrect thing');
        }
    "#
        ),
        Contract::from_code_path(
            "MockChecker".to_string(),
            Path::new("tests/data/contracts/mock_checker.cairo"),
        )
        .unwrap()
    );

    let result = run(
        &test.path().unwrap(),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();
    assert_passed!(result);
}

#[test]
fn mock_call_proxy() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use array::ArrayTrait;
        use starknet::ContractAddress;
        use cheatcodes::{ declare, PreparedContract, deploy, start_mock_call };

        #[starknet::interface]
        trait IMockCheckerProxy<TContractState> {
            fn get_thing_from_contract(ref self: TContractState, address: ContractAddress) -> felt252;
        }

        #[test]
        fn mock_call_proxy() {
            let calldata = array![420];

            let class_hash = declare('MockChecker');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let mock_checker_contract_address = deploy(prepared).unwrap();
            let mock_ret_data = array![421];
            start_mock_call(mock_checker_contract_address, 'get_thing', mock_ret_data);

            let class_hash = declare('MockCheckerProxy');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
            let proxy_contract_address = deploy(prepared).unwrap();
            let proxy_dispatcher = IMockCheckerProxyDispatcher { contract_address: proxy_contract_address };
            let thing = proxy_dispatcher.get_thing_from_contract(mock_checker_contract_address);

            assert(thing == 421, 'Incorrect thing');
        }
    "#
        ),
        Contract::from_code_path(
            "MockChecker".to_string(),
            Path::new("tests/data/contracts/mock_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "MockCheckerProxy".to_string(),
            Path::new("tests/data/contracts/mock_checker_proxy.cairo"),
        )
        .unwrap()
    );

    let result = run(
        &test.path().unwrap(),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();
    assert_passed!(result);
}

#[test]
fn mock_call_two_methods() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use cheatcodes::{ declare, PreparedContract, deploy, start_mock_call };

        #[starknet::interface]
        trait IMockChecker<TContractState> {
            fn get_thing(ref self: TContractState) -> felt252;
            fn get_constant_thing(ref self: TContractState) -> felt252;
        }

        #[test]
        fn mock_call_two_methods() {
            let calldata = array![420];

            let class_hash = declare('MockChecker');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let contract_address = deploy(prepared).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mock_ret_data = array![421];
            start_mock_call(contract_address, 'get_thing', mock_ret_data);

            let mock_ret_data = array![999];
            start_mock_call(contract_address, 'get_constant_thing', mock_ret_data);

            let thing = dispatcher.get_thing();
            let constant_thing = dispatcher.get_constant_thing();

            assert(thing == 421, 'Incorrect thing');
            assert(constant_thing == 999, 'Incorrect other thing');
        }
    "#
        ),
        Contract::from_code_path(
            "MockChecker".to_string(),
            Path::new("tests/data/contracts/mock_checker.cairo"),
        )
        .unwrap()
    );

    let result = run(
        &test.path().unwrap(),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();
    assert_passed!(result);
}

#[test]
fn start_mock_call_inner_call_has_no_effect() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use cheatcodes::{ declare, PreparedContract, deploy, start_mock_call };

        #[starknet::interface]
        trait IMockChecker<TContractState> {
            fn get_thing_wrapper(ref self: TContractState) -> felt252;
        }

        #[test]
        fn start_mock_call_inner_call_has_no_effect() {
            let calldata = array![420];

            let class_hash = declare('MockChecker');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let contract_address = deploy(prepared).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mock_ret_data = array![421];
            start_mock_call(contract_address, 'get_thing', mock_ret_data);

            let thing = dispatcher.get_thing_wrapper();

            assert(thing == 420, 'Incorrect thing');
        }
    "#
        ),
        Contract::from_code_path(
            "MockChecker".to_string(),
            Path::new("tests/data/contracts/mock_checker.cairo"),
        )
        .unwrap()
    );

    let result = run(
        &test.path().unwrap(),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();
    assert_passed!(result);
}

#[test]
fn start_mock_call_with_library_call_has_no_effect() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use cheatcodes::{ declare, PreparedContract, deploy, start_mock_call };
            use starknet::ClassHash;

            #[starknet::interface]
            trait IMockCheckerLibCall<TContractState> {
                fn get_constant_thing_with_lib_call(ref self: TContractState, class_hash: ClassHash) -> felt252;
            }

            #[test]
            fn start_mock_call_with_library_call_has_no_effect() {
                let mock_checker_class_hash = declare('MockChecker');

                let class_hash = declare('MockCheckerLibCall');
                let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
                let contract_address = deploy(prepared).unwrap();

                let mock_ret_data = array![421];
                start_mock_call(contract_address, 'get_thing', mock_ret_data);

                let dispatcher = IMockCheckerLibCallDispatcher { contract_address };
                let thing = dispatcher.get_constant_thing_with_lib_call(mock_checker_class_hash);
                assert(thing == 13, thing);
            }
        "#
        ),
        Contract::from_code_path(
            "MockChecker".to_string(),
            Path::new("tests/data/contracts/mock_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "MockCheckerLibCall".to_string(),
            Path::new("tests/data/contracts/mock_checker_library_call.cairo"),
        )
        .unwrap()
    );

    let result = run(
        &test.path().unwrap(),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

    assert_passed!(result);
}

#[test]
fn start_mock_call_in_constructor_test_has_no_effect() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use array::ArrayTrait;
        use cheatcodes::{ declare, PreparedContract, deploy, start_mock_call };

        #[starknet::interface]
        trait IConstructorMockChecker<TContractState> {
            fn get_stored_thing(ref self: TContractState) -> felt252;
            fn get_constant_thing(ref self: TContractState) -> felt252;
        }

        #[test]
        fn start_mock_call_in_constructor_test_has_no_effect() {
            let class_hash = declare('ConstructorMockChecker');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
            let contract_address = deploy(prepared).unwrap();

            let dispatcher = IConstructorMockCheckerDispatcher { contract_address };

            let mock_ret_data = array![421];
            start_mock_call(contract_address, 'get_constant_thing', mock_ret_data);

            let thing = dispatcher.get_stored_thing();

            assert(thing == 13, 'Incorrect thing');
        }
    "#
        ),
        Contract::from_code_path(
            "ConstructorMockChecker".to_string(),
            Path::new("tests/data/contracts/constructor_mock_checker.cairo"),
        )
        .unwrap()
    );

    let result = run(
        &test.path().unwrap(),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();
    assert_passed!(result);
}

#[test]
fn start_mock_call_with_syscall_has_no_effect() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use cheatcodes::{ declare, PreparedContract, deploy, start_mock_call };

        #[starknet::interface]
        trait IMockChecker<TContractState> {
            fn get_thing_and_emit_event(ref self: TContractState) -> felt252;
        }

        #[test]
        fn mock_call_with_syscall_has_no_effect() {
            let calldata = array![420];

            let class_hash = declare('MockChecker');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let contract_address = deploy(prepared).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mock_ret_data = array![421];
            start_mock_call(contract_address, 'get_thing', mock_ret_data);

            let thing = dispatcher.get_thing_and_emit_event();

            assert(thing == 420, 'Incorrect thing');
        }
    "#
        ),
        Contract::from_code_path(
            "MockChecker".to_string(),
            Path::new("tests/data/contracts/mock_checker.cairo"),
        )
        .unwrap()
    );

    let result = run(
        &test.path().unwrap(),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();
    assert_passed!(result);
}