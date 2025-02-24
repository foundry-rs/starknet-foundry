use indoc::indoc;
use std::path::Path;
use test_utils::runner::{assert_passed, Contract};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
fn mock_call_simple() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, start_mock_call, stop_mock_call };

        #[starknet::interface]
        trait IMockChecker<TContractState> {
            fn get_thing(ref self: TContractState) -> felt252;
        }

        #[test]
        fn mock_call_simple() {
            let calldata = array![420];

            let contract = declare("MockChecker").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@calldata).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mock_ret_data = 421;

            start_mock_call(contract_address, selector!("get_thing"), mock_ret_data);
            let thing = dispatcher.get_thing();
            assert(thing == 421, 'Incorrect thing');

            stop_mock_call(contract_address, selector!("get_thing"));
            let thing = dispatcher.get_thing();
            assert(thing == 420, 'Incorrect thing');
        }

        #[test]
        fn mock_call_simple_before_dispatcher_created() {
            let calldata = array![420];

            let contract = declare("MockChecker").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@calldata).unwrap();

            let mock_ret_data = 421;
            start_mock_call(contract_address, selector!("get_thing"), mock_ret_data);

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

    let result = run_test_case(&test);
    assert_passed(&result);
}

#[test]
fn mock_call_complex_types() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use array::ArrayTrait;
        use serde::Serde;
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, start_mock_call };

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

            let contract = declare("MockChecker").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@calldata).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mock_ret_data = StructThing {item_one: 412, item_two: 421};
            start_mock_call(contract_address, selector!("get_struct_thing"), mock_ret_data);

            let thing: StructThing = dispatcher.get_struct_thing();

            assert(thing.item_one == 412, 'thing.item_one');
            assert(thing.item_two == 421, 'thing.item_two');
        }

        #[test]
        fn start_mock_call_return_arr() {
            let calldata = array![420];

            let contract = declare("MockChecker").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@calldata).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mock_ret_data =  array![ StructThing {item_one: 112, item_two: 121}, StructThing {item_one: 412, item_two: 421} ];
            start_mock_call(contract_address, selector!("get_arr_thing"), mock_ret_data);

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

    let result = run_test_case(&test);
    assert_passed(&result);
}

#[test]
fn mock_calls() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, mock_call, start_mock_call, stop_mock_call };

        #[starknet::interface]
        trait IMockChecker<TContractState> {
            fn get_thing(ref self: TContractState) -> felt252;
        }

        #[test]
        fn mock_call_one() {
            let calldata = array![420];

            let contract = declare("MockChecker").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@calldata).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mock_ret_data = 421;

            mock_call(contract_address, selector!("get_thing"), mock_ret_data, 1);

            let thing = dispatcher.get_thing();
            assert_eq!(thing, 421);

            let thing = dispatcher.get_thing();
            assert_eq!(thing, 420);
        }

        #[test]
        fn mock_call_twice() {
            let calldata = array![420];

            let contract = declare("MockChecker").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@calldata).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mock_ret_data = 421;

            mock_call(contract_address, selector!("get_thing"), mock_ret_data, 2);

            let thing = dispatcher.get_thing();
            assert_eq!(thing, 421);

            let thing = dispatcher.get_thing();
            assert_eq!(thing, 421);

            let thing = dispatcher.get_thing();
            assert_eq!(thing, 420);
        }
    "#
        ),
        Contract::from_code_path(
            "MockChecker".to_string(),
            Path::new("tests/data/contracts/mock_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);
    assert_passed(&result);
}

#[test]
fn mock_call_when_simple() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, start_mock_call_when, stop_mock_call_when, MockCalldata };

        #[starknet::interface]
        trait IMockChecker<TContractState> {
            fn get_thing(ref self: TContractState) -> felt252;
        }

        #[test]
        fn mock_call_when_simple() {
            let calldata = array![420];

            let contract = declare("MockChecker").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@calldata).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };
            
            let specific_mock_ret_data = 421;
            let default_mock_ret_data = 404;
            let expected_calldata = MockCalldata::Values([].span());

            start_mock_call_when(contract_address, selector!("get_thing"), expected_calldata, specific_mock_ret_data);
            start_mock_call_when(contract_address, selector!("get_thing"), MockCalldata::Any, default_mock_ret_data);
            let thing = dispatcher.get_thing();
            assert(thing == specific_mock_ret_data, 'Incorrect thing');

            stop_mock_call_when(contract_address, selector!("get_thing"), expected_calldata);
            let thing = dispatcher.get_thing();
            assert(thing == default_mock_ret_data, 'Incorrect thing');

            stop_mock_call_when(contract_address, selector!("get_thing"), MockCalldata::Any);
            let thing = dispatcher.get_thing();
            assert(thing == 420, 'Incorrect thing');
        }

        #[test]
        fn mock_call_when_simple_before_dispatcher_created() {
            let calldata = array![420];

            let contract = declare("MockChecker").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@calldata).unwrap();

            let specific_mock_ret_data = 421;
            let default_mock_ret_data = 404;
            let expected_calldata = MockCalldata::Values([].span());
            
            start_mock_call_when(contract_address, selector!("get_thing"), expected_calldata, specific_mock_ret_data);
            start_mock_call_when(contract_address, selector!("get_thing"), MockCalldata::Any, default_mock_ret_data);
            let dispatcher = IMockCheckerDispatcher { contract_address };
            let thing = dispatcher.get_thing();

            assert(thing == specific_mock_ret_data, 'Incorrect thing');

            stop_mock_call_when(contract_address, selector!("get_thing"), expected_calldata);
            let thing = dispatcher.get_thing();
            assert(thing == default_mock_ret_data, 'Incorrect thing');

            stop_mock_call_when(contract_address, selector!("get_thing"), MockCalldata::Any);
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

    let result = run_test_case(&test);
    assert_passed(&result);
}

#[test]
fn mock_call_when_complex_types() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use array::ArrayTrait;
        use serde::Serde;
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, start_mock_call_when, stop_mock_call_when, MockCalldata };

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
        fn start_mock_call_when_return_struct() {
            let calldata = array![420];

            let contract = declare("MockChecker").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@calldata).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let default_mock_ret_data = StructThing {item_one: 412, item_two: 421};
            let specific_mock_ret_data = StructThing {item_one: 404, item_two: 401};
            let expected_calldata = MockCalldata::Values([].span());

            start_mock_call_when(contract_address, selector!("get_struct_thing"), MockCalldata::Any, default_mock_ret_data);
            start_mock_call_when(contract_address, selector!("get_struct_thing"), expected_calldata, specific_mock_ret_data);

            let thing: StructThing = dispatcher.get_struct_thing();

            assert(thing.item_one == 404, 'thing.item_one');
            assert(thing.item_two == 401, 'thing.item_two');

            stop_mock_call_when(contract_address, selector!("get_struct_thing"), expected_calldata);
            let thing: StructThing = dispatcher.get_struct_thing();

            assert(thing.item_one == 412, 'thing.item_one');
            assert(thing.item_two == 421, 'thing.item_two');
        }

        #[test]
        fn start_mock_call_when_return_arr() {
            let calldata = array![420];

            let contract = declare("MockChecker").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@calldata).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let default_mock_ret_data =  array![ StructThing {item_one: 112, item_two: 121}, StructThing {item_one: 412, item_two: 421} ];
            let specific_mock_ret_data =  array![ StructThing {item_one: 212, item_two: 221}, StructThing {item_one: 512, item_two: 521} ];

            let expected_calldata = MockCalldata::Values([].span());

            start_mock_call_when(contract_address, selector!("get_arr_thing"), MockCalldata::Any, default_mock_ret_data);
            start_mock_call_when(contract_address, selector!("get_arr_thing"), expected_calldata, specific_mock_ret_data);

            let things: Array<StructThing> = dispatcher.get_arr_thing();

            let thing = things.at(0);
            assert(*thing.item_one == 212, 'thing1.item_one 1');
            assert(*thing.item_two == 221, 'thing1.item_two');

            let thing = things.at(1);
            assert(*thing.item_one == 512, 'thing2.item_one 2');
            assert(*thing.item_two == 521, 'thing2.item_two');

            stop_mock_call_when(contract_address, selector!("get_arr_thing"), expected_calldata);

            let things: Array<StructThing> = dispatcher.get_arr_thing();

            let thing = things.at(0);
            assert(*thing.item_one == 112, 'thing1.item_one 3');
            assert(*thing.item_two == 121, 'thing1.item_two');

            let thing = things.at(1);
            assert(*thing.item_one == 412, 'thing2.item_one 4');
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

    let result = run_test_case(&test);
    assert_passed(&result);
}

#[test]
fn mock_calls_when() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, mock_call_when, MockCalldata};

        #[starknet::interface]
        trait IMockChecker<TContractState> {
            fn get_thing(ref self: TContractState) -> felt252;
        }

        #[test]
        fn mock_call_when_one_specific() {
            let calldata = array![420];

            let contract = declare("MockChecker").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@calldata).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mock_ret_data = 421;
            let expected_calldata = MockCalldata::Values([].span());
            mock_call_when(contract_address, selector!("get_thing"), expected_calldata, mock_ret_data, 1);

            let thing = dispatcher.get_thing();
            assert_eq!(thing, 421);

            let thing = dispatcher.get_thing();
            assert_eq!(thing, 420);
        }

        #[test]
        fn mock_call_when_twice_specific() {
            let calldata = array![420];

            let contract = declare("MockChecker").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@calldata).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mock_ret_data = 421;
            let expected_calldata = MockCalldata::Values([].span());
            mock_call_when(contract_address, selector!("get_thing"), expected_calldata, mock_ret_data, 2);

            let thing = dispatcher.get_thing();
            assert_eq!(thing, 421);

            let thing = dispatcher.get_thing();
            assert_eq!(thing, 421);

            let thing = dispatcher.get_thing();
            assert_eq!(thing, 420);
        }

        #[test]
        fn mock_call_when_one_any() {
            let calldata = array![420];

            let contract = declare("MockChecker").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@calldata).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mock_ret_data = 421;
            mock_call_when(contract_address, selector!("get_thing"), MockCalldata::Any, mock_ret_data, 1);

            let thing = dispatcher.get_thing();
            assert_eq!(thing, 421);

            let thing = dispatcher.get_thing();
            assert_eq!(thing, 420);
        }

        #[test]
        fn mock_call_when_twice_any() {
            let calldata = array![420];

            let contract = declare("MockChecker").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@calldata).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mock_ret_data = 421;
            mock_call_when(contract_address, selector!("get_thing"), MockCalldata::Any, mock_ret_data, 2);

            let thing = dispatcher.get_thing();
            assert_eq!(thing, 421);

            let thing = dispatcher.get_thing();
            assert_eq!(thing, 421);

            let thing = dispatcher.get_thing();
            assert_eq!(thing, 420);
        }

    "#
        ),
        Contract::from_code_path(
            "MockChecker".to_string(),
            Path::new("tests/data/contracts/mock_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);
    assert_passed(&result);
}

#[test]
fn mock_calls_when_mixed() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, mock_call_when, MockCalldata};

        #[starknet::interface]
        trait IMockChecker<TContractState> {
            fn get_thing(ref self: TContractState) -> felt252;
        }

        #[test]
        fn mock_call_when_one() {
            let calldata = array![420];

            let contract = declare("MockChecker").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@calldata).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mock_ret_data = 421;
            let expected_calldata = MockCalldata::Values([].span());
            mock_call_when(contract_address, selector!("get_thing"), expected_calldata, mock_ret_data, 1);
            mock_call_when(contract_address, selector!("get_thing"), MockCalldata::Any, 422, 1);
            
            let thing = dispatcher.get_thing();
            assert_eq!(thing, 421, "Specific calldata");

            let thing = dispatcher.get_thing();
            assert_eq!(thing, 422, "Any calldata");

            let thing = dispatcher.get_thing();
            assert_eq!(thing, 420);
        }

        #[test]
        fn mock_call_when_multi() {
            let calldata = array![420];

            let contract = declare("MockChecker").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@calldata).unwrap();

            let dispatcher = IMockCheckerDispatcher { contract_address };

            let mock_ret_data = 421;
            let expected_calldata = MockCalldata::Values([].span());
            mock_call_when(contract_address, selector!("get_thing"), expected_calldata, mock_ret_data, 3);
            mock_call_when(contract_address, selector!("get_thing"), MockCalldata::Any, 422, 2);

            let thing = dispatcher.get_thing();
            assert_eq!(thing, 421, "1st Specific calldata");

            let thing = dispatcher.get_thing();
            assert_eq!(thing, 421, "2nd Specific calldata");

            let thing = dispatcher.get_thing();
            assert_eq!(thing, 421, "3rd Specific calldata");

            let thing = dispatcher.get_thing();
            assert_eq!(thing, 422, "1st Any calldata");

            let thing = dispatcher.get_thing();
            assert_eq!(thing, 422, "2nd Any calldata");

            let thing = dispatcher.get_thing();
            assert_eq!(thing, 420);
        }
    "#
        ),
        Contract::from_code_path(
            "MockChecker".to_string(),
            Path::new("tests/data/contracts/mock_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);
    assert_passed(&result);
}
