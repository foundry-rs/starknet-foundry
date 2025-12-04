use core::array::ArrayTrait;
use core::panic_with_felt252;
use should_panic_test::constructor_panic::{
    IConstructorPanickingContractProxyDispatcher, IConstructorPanickingContractProxyDispatcherTrait,
};
use snforge_std::{ContractClassTrait, DeclareResult, DeclareResultTrait, declare};

#[test]
#[should_panic]
fn should_panic_no_data() {
    panic_with_felt252(0);
}

#[test]
#[should_panic(expected: ('panic message',))]
fn should_panic_check_data() {
    panic_with_felt252('panic message');
}

#[test]
#[should_panic(expected: ('panic message', 'second message'))]
fn should_panic_multiple_messages() {
    let mut arr = ArrayTrait::new();
    arr.append('panic message');
    arr.append('second message');
    panic(arr);
}

#[test]
#[should_panic(expected: (0,))]
fn should_panic_with_non_matching_data() {
    panic_with_felt252('failing check');
}

#[test]
fn didnt_expect_panic() {
    panic_with_felt252('unexpected panic');
}

#[test]
#[should_panic]
fn expected_panic_but_didnt() {
    assert(1 == 1, 'err');
}

#[test]
#[should_panic(expected: 'panic message')]
fn expected_panic_but_didnt_with_expected() {
    assert(1 == 1, 'err');
}

#[test]
#[should_panic(expected: ('panic message', 'second message'))]
fn expected_panic_but_didnt_with_expected_multiple() {
    assert(1 == 1, 'err');
}

#[test]
#[should_panic(expected: 'panic message')]
fn should_panic_felt_matching() {
    assert(1 != 1, 'panic message');
}

#[test]
#[should_panic(expected: "will panicc")]
fn should_panic_not_matching_suffix() {
    panic!("This will panic");
}

#[test]
#[should_panic(expected: "will panic")]
fn should_panic_match_suffix() {
    panic!("This will panic");
}


#[test]
#[should_panic(expected: ('This will panic',))]
fn should_panic_byte_array_with_felt() {
    panic!("This will panic");
}

#[test]
#[should_panic(expected: "This will panic")]
fn should_panic_felt_with_byte_array() {
    panic_with_felt252('This will panic');
}

#[test]
#[should_panic(expected: "This will panic")]
fn should_panic_expected_contains_error() {
    panic!("will");
}

#[test]
#[should_panic]
fn deployment_with_panic_not_possible_to_catch() {
    let contract_class = declare("ConstructorPanickingContract").unwrap().contract_class();
    match contract_class.deploy(@array![]) {
        // Unreachable
        Result::Ok(_) => {},
        Result::Err(_) => {},
    }
}

#[test]
#[should_panic]
fn proxied_deployment_with_panic_not_possible_to_catch() {
    let proxy = declare("ConstructorPanickingContractProxy").unwrap().contract_class();
    let (proxy_address, _) = proxy.deploy(@array![]).unwrap();

    let dispatcher = IConstructorPanickingContractProxyDispatcher {
        contract_address: proxy_address,
    };

    let declare_result = declare("ConstructorPanickingContract").unwrap();
    let class_hash = match declare_result {
        DeclareResult::AlreadyDeclared(res) => res.class_hash,
        DeclareResult::Success(res) => res.class_hash,
    };

    match dispatcher.deploy_constructor_panicking_contract(class_hash) {
        // Unreachable
        Result::Ok(_) => {},
        Result::Err(_) => {},
    }
}

