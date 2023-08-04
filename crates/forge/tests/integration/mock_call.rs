use crate::integration::common::corelib::{corelib, predeployed_contracts};
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
        use cheatcodes::PreparedContract;
        use array::ArrayTrait;

        #[starknet::interface]
        trait IMockChecker<TContractState> {
            fn get_thing(self: @TContractState) -> felt252;
        }

        #[test]
        fn start_mock_call_simple() {
            let mut calldata = ArrayTrait::new();
            calldata.append(420);

            let class_hash = declare('MockChecker');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let contract_address = deploy(prepared).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mut mock_ret_data = ArrayTrait::new();
            mock_ret_data.append(421);
            start_mock_call(contract_address, 'get_thing', mock_ret_data);

            let thing = dispatcher.get_thing();

            assert(thing == 421, 'Incorrect thing');
        }

        #[test]
        fn start_mock_call_simple_mock_before_dispatcher_created() {
            let mut calldata = ArrayTrait::new();
            calldata.append(420);

            let class_hash = declare('MockChecker');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let contract_address = deploy(prepared).unwrap();

            let mut mock_ret_data = ArrayTrait::new();
            mock_ret_data.append(421);
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
        Some(&Utf8PathBuf::from_path_buf(corelib().to_path_buf()).unwrap()),
        &test.contracts(corelib().path()).unwrap(),
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
        use cheatcodes::PreparedContract;
        use array::ArrayTrait;

        #[starknet::interface]
        trait IMockChecker<TContractState> {
            fn get_thing(self: @TContractState) -> felt252;
        }

        #[test]
        fn stop_mock_call_simple() {
            let mut calldata = ArrayTrait::new();
            calldata.append(420);

            let class_hash = declare('MockChecker');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let contract_address = deploy(prepared).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mut mock_ret_data = ArrayTrait::new();
            mock_ret_data.append(421);

            start_mock_call(contract_address, 'get_thing', mock_ret_data);
            let thing = dispatcher.get_thing();
            assert(thing == 421, 'Incorrect thing');

            stop_mock_call(contract_address, 'get_thing');
            let thing = dispatcher.get_thing();
            assert(thing == 420, 'Incorrect thing');
        }

        #[test]
        fn stop_mock_call_when_mock_not_started() {
            let mut calldata = ArrayTrait::new();
            calldata.append(420);

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
        Some(&Utf8PathBuf::from_path_buf(corelib().to_path_buf()).unwrap()),
        &test.contracts(corelib().path()).unwrap(),
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
        use cheatcodes::PreparedContract;
        use array::ArrayTrait;

        #[starknet::interface]
        trait IMockChecker<TContractState> {
            fn get_thing(self: @TContractState) -> felt252;
        }

        #[test]
        fn mock_call_double_mocks() {
            let mut calldata = ArrayTrait::new();
            calldata.append(420);

            let class_hash = declare('MockChecker');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let contract_address = deploy(prepared).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mut mock_ret_data = ArrayTrait::new();
            mock_ret_data.append(421);
            start_mock_call(contract_address, 'get_thing', mock_ret_data);

            let mut mock_ret_data = ArrayTrait::new();
            mock_ret_data.append(427);
            start_mock_call(contract_address, 'get_thing', mock_ret_data);

            let thing = dispatcher.get_thing();
            assert(thing == 427, 'Incorrect thing');

            stop_mock_call(contract_address, 'get_thing');
            let thing = dispatcher.get_thing();
            assert(thing == 420, 'Incorrect thing');
        }

        #[test]
        fn mock_call_double_calls() {
            let mut calldata = ArrayTrait::new();
            calldata.append(420);

            let class_hash = declare('MockChecker');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let contract_address = deploy(prepared).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mut mock_ret_data = ArrayTrait::new();
            mock_ret_data.append(421);
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
        Some(&Utf8PathBuf::from_path_buf(corelib().to_path_buf()).unwrap()),
        &test.contracts(corelib().path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();
    assert_passed!(result);
}

#[test]
#[should_panic]
fn mock_call_inner_call() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use cheatcodes::PreparedContract;
        use array::ArrayTrait;

        #[starknet::interface]
        trait IMockChecker<TContractState> {
            fn get_thing_wrapper(self: @TContractState) -> felt252;
        }

        #[test]
        fn mock_call_inner() {
            let mut calldata = ArrayTrait::new();
            calldata.append(420);

            let class_hash = declare('MockChecker');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let contract_address = deploy(prepared).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mut mock_ret_data = ArrayTrait::new();
            mock_ret_data.append(421);
            start_mock_call(contract_address, 'get_thing', mock_ret_data);

            let thing = dispatcher.get_thing_wrapper();

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
        Some(&Utf8PathBuf::from_path_buf(corelib().to_path_buf()).unwrap()),
        &test.contracts(corelib().path()).unwrap(),
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
        use cheatcodes::PreparedContract;
        use array::ArrayTrait;
        use starknet::ContractAddress;

        #[starknet::interface]
        trait IMockCheckerProxy<TContractState> {
            fn get_thing_from_contract(ref self: TContractState, address: ContractAddress) -> felt252;
        }

        #[test]
        fn mock_call_proxy() {
            let mut calldata = ArrayTrait::new();
            calldata.append(420);

            let class_hash = declare('MockChecker');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let mock_checker_contract_address = deploy(prepared).unwrap();
            let mut mock_ret_data = ArrayTrait::new();
            mock_ret_data.append(421);
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
        Some(&Utf8PathBuf::from_path_buf(corelib().to_path_buf()).unwrap()),
        &test.contracts(corelib().path()).unwrap(),
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
        use cheatcodes::PreparedContract;
        use array::ArrayTrait;

        #[starknet::interface]
        trait IMockChecker<TContractState> {
            fn get_thing(self: @TContractState) -> felt252;
            fn get_other_thing(self: @TContractState) -> felt252;
        }

        #[test]
        fn mock_call_two_methods() {
            let mut calldata = ArrayTrait::new();
            calldata.append(420);

            let class_hash = declare('MockChecker');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let contract_address = deploy(prepared).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mut mock_ret_data = ArrayTrait::new();
            mock_ret_data.append(421);
            start_mock_call(contract_address, 'get_thing', mock_ret_data);

            let mut mock_ret_data = ArrayTrait::new();
            mock_ret_data.append(999);
            start_mock_call(contract_address, 'get_other_thing', mock_ret_data);

            let thing = dispatcher.get_thing();
            let other_thing = dispatcher.get_other_thing();

            assert(thing == 421, 'Incorrect thing');
            assert(other_thing == 999, 'Incorrect other thing');
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
        Some(&Utf8PathBuf::from_path_buf(corelib().to_path_buf()).unwrap()),
        &test.contracts(corelib().path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();
    assert_passed!(result);
}
