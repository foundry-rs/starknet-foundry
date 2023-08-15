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
        use snforge_std::{ declare, PreparedContract, deploy, start_mock_call };

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

            let mock_ret_data = 421;
            start_mock_call(contract_address, 'get_thing', mock_ret_data);

            let thing = dispatcher.get_thing();

            assert(thing == 421, thing);
        }

        #[test]
        fn start_mock_call_simple_mock_before_dispatcher_created() {
            let calldata = array![420];

            let class_hash = declare('MockChecker');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let contract_address = deploy(prepared).unwrap();

            let mock_ret_data = 421;
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
        &String::from("src"),
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
fn start_mock_call_return_complex_dtypes() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use array::ArrayTrait;
        use serde::Serde;
        use snforge_std::{ declare, PreparedContract, deploy, start_mock_call };

        #[starknet::interface]
        trait IMockChecker<TContractState> {
            fn get_struct_thing(ref self: TContractState) -> StructThing;
            fn get_arr_thing(ref self: TContractState) -> Array<StructThing>;
        }

        #[derive(Serde, Drop)]
        struct StructThing {
            item_one: felt252,
            item_two: felt252,
        }

        #[test]
        fn start_mock_call_return_struct() {
            let calldata = array![420];

            let class_hash = declare('MockChecker');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let contract_address = deploy(prepared).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mock_ret_data = StructThing {item_one: 412, item_two: 421};
            start_mock_call(contract_address, 'get_struct_thing', mock_ret_data);

            let thing: StructThing = dispatcher.get_struct_thing();

            assert(thing.item_one == 412, 'thing.item_one');
            assert(thing.item_two == 421, 'thing.item_two');
        }

        #[test]
        fn start_mock_call_return_arr() {
            let calldata = array![420];

            let class_hash = declare('MockChecker');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let contract_address = deploy(prepared).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mock_ret_data =  array![ StructThing {item_one: 112, item_two: 121}, StructThing {item_one: 412, item_two: 421} ];
            start_mock_call(contract_address, 'get_arr_thing', mock_ret_data);

            let things: Array<StructThing> = dispatcher.get_arr_thing();

            let thing = things.at(0);
            assert(*thing.item_one == 112, 'thing1.item_one');
            assert(*thing.item_two == 121, 'thing1.item_two');

            let thing = things.at(1);
            assert(*thing.item_one == 412, 'thing2.item_one');
            assert(*thing.item_two == 421, 'thing2.item_two');
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
        &String::from("src"),
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
        use snforge_std::{ declare, PreparedContract, deploy, start_mock_call, stop_mock_call };

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

            let mock_ret_data = 421;

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
        &String::from("src"),
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
        use snforge_std::{ declare, PreparedContract, deploy, start_mock_call, stop_mock_call };

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

            let mock_ret_data = 421;
            start_mock_call(contract_address, 'get_thing', mock_ret_data);

            let  mock_ret_data = 427;
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

            let mock_ret_data = 421;
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
        &String::from("src"),
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
        use snforge_std::{ declare, PreparedContract, deploy, start_mock_call };

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
            let mock_ret_data = 421;
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
        &String::from("src"),
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
        use snforge_std::{ declare, PreparedContract, deploy, start_mock_call };

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

            let mock_ret_data = 421;
            start_mock_call(contract_address, 'get_thing', mock_ret_data);

            let mock_ret_data = 999;
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
        &String::from("src"),
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
#[ignore]
fn start_mock_call_in_constructor_test() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use array::ArrayTrait;
        use snforge_std::{ declare, PreparedContract, deploy, start_mock_call };

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

            let mock_ret_data = 421;
            start_mock_call(contract_address, 'get_constant_thing', mock_ret_data);

            let thing = dispatcher.get_stored_thing();

            assert(thing == 421, 'Incorrect thing');
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
        &String::from("src"),
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
fn start_mock_call_with_syscall() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use starknet::ContractAddress;
        use snforge_std::{ declare, PreparedContract, deploy, start_mock_call, stop_mock_call };

        #[starknet::interface]
        trait IMockCheckerProxy<TContractState> {
            fn get_thing_from_contract_and_emit_event(ref self: TContractState, address: ContractAddress) -> felt252;
        }

        #[test]
        fn start_mock_call_with_syscall() {
            let calldata = array![420];

            let class_hash = declare('MockChecker');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let mock_checker_contract_address = deploy(prepared).unwrap();
            let mock_ret_data = 421;
            start_mock_call(mock_checker_contract_address, 'get_thing', mock_ret_data);

            let class_hash = declare('MockCheckerProxy');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
            let proxy_contract_address = deploy(prepared).unwrap();
            let proxy_dispatcher = IMockCheckerProxyDispatcher { contract_address: proxy_contract_address };
            let thing = proxy_dispatcher.get_thing_from_contract_and_emit_event(mock_checker_contract_address);

            assert(thing == 421, 'Incorrect thing');

            stop_mock_call(mock_checker_contract_address, 'get_thing');
            let thing = proxy_dispatcher.get_thing_from_contract_and_emit_event(mock_checker_contract_address);
            assert(thing == 420, 'Incorrect thing');
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
        &String::from("src"),
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
        use snforge_std::{ declare, PreparedContract, deploy, start_mock_call };

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

            let mock_ret_data = 421;
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
        &String::from("src"),
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
            use snforge_std::{ declare, PreparedContract, deploy, start_mock_call };
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

                let mock_ret_data = 421;
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
        &String::from("src"),
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
fn start_mock_call_when_contract_not_deployed() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use option::OptionTrait;
        use traits::TryInto;
        use starknet::ContractAddress;
        use starknet::Felt252TryIntoContractAddress;
        use snforge_std::{ declare, PreparedContract, deploy, start_mock_call };

        #[starknet::interface]
        trait IMockChecker<TContractState> {
            fn get_thing(ref self: TContractState) -> felt252;
        }

        #[test]
        fn start_mock_call_when_contract_not_deployed() {
            let calldata = array![420];

            let class_hash = declare('MockChecker');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };

            let contract_address: felt252 = 123;
            let contract_address: ContractAddress = contract_address.try_into().unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mock_ret_data = 421;
            start_mock_call(contract_address, 'get_thing', mock_ret_data);
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
        &String::from("src"),
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
fn start_mock_call_when_function_not_implemented() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use option::OptionTrait;
        use traits::TryInto;
        use starknet::ContractAddress;
        use starknet::Felt252TryIntoContractAddress;
        use snforge_std::{ declare, PreparedContract, deploy, start_mock_call };

        #[starknet::interface]
        trait IMockChecker<TContractState> {
            fn get_thing_not_implemented(ref self: TContractState) -> felt252;
        }

        #[test]
        fn start_mock_call_when_function_not_implemented() {
            let calldata = array![420];

            let class_hash = declare('MockChecker');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let contract_address = deploy(prepared).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mock_ret_data = 421;
            start_mock_call(contract_address, 'get_thing_not_implemented', mock_ret_data);
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
        &String::from("src"),
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
