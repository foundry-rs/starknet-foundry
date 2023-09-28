use indoc::indoc;
use std::path::Path;
use utils::runner::Contract;
use utils::running_tests::run_test_case;
use utils::{assert_passed, test_case};

#[test]
fn mock_call_simple() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use snforge_std::{ declare, ContractClassTrait, start_mock_call, stop_mock_call };

        #[starknet::interface]
        trait IMockChecker<TContractState> {
            fn get_thing(ref self: TContractState) -> felt252;
        }

        #[test]
        fn mock_call_simple() {
            let calldata = array![420];

            let contract = declare('MockChecker');
            let contract_address = contract.deploy(@calldata).unwrap();

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
        fn mock_call_before_dispatcher_created() {
            let calldata = array![420];

            let contract = declare('MockChecker');
            let contract_address = contract.deploy(@calldata).unwrap();

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

    let result = run_test_case(&test);
    assert_passed!(result);
}

#[test]
fn mock_call_complex_dtypes() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use array::ArrayTrait;
        use serde::Serde;
        use snforge_std::{ declare, ContractClassTrait, start_mock_call };

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

            let contract = declare('MockChecker');
            let contract_address = contract.deploy(@calldata).unwrap();

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

            let contract = declare('MockChecker');
            let contract_address = contract.deploy(@calldata).unwrap();

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

    let result = run_test_case(&test);
    assert_passed!(result);
}
