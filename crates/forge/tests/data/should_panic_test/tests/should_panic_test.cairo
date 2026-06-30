use core::array::ArrayTrait;
use core::byte_array::BYTE_ARRAY_MAGIC;
use core::panic_with_felt252;
use should_panic_test::{IPanickingDispatcher, IPanickingDispatcherTrait};
use snforge_std::{ContractClassTrait, DeclareResultTrait, declare};

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
#[should_panic(expected: ("this_string_is_longer_than_31_bytes", 11, "hello", 5, 'short_string'))]
fn should_panic_mixed_tuple() {
    let mut arr = ArrayTrait::new();
    let first = "this_string_is_longer_than_31_bytes";
    let second = "hello";

    arr.append(BYTE_ARRAY_MAGIC);
    Serde::<ByteArray>::serialize(@first, ref arr);
    arr.append(11);
    arr.append(BYTE_ARRAY_MAGIC);
    Serde::<ByteArray>::serialize(@second, ref arr);
    arr.append(5);
    arr.append('short_string');
    panic(arr);
}

#[test]
#[should_panic(expected: ("this_string_is_longer_than_31_bytes", 11, "helloo", 5, 'short_string'))]
fn should_panic_mixed_tuple_with_non_matching_data() {
    let mut arr = ArrayTrait::new();
    let first = "this_string_is_longer_than_31_bytes";
    let second = "hello";

    arr.append(BYTE_ARRAY_MAGIC);
    Serde::<ByteArray>::serialize(@first, ref arr);
    arr.append(11);
    arr.append(BYTE_ARRAY_MAGIC);
    Serde::<ByteArray>::serialize(@second, ref arr);
    arr.append(5);
    arr.append('short_string');
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
#[should_panic(expected: "will panic")]
fn should_panic_propagated_byte_array_substring() {
    let contract = declare("PanickingContract").unwrap().contract_class();
    let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();

    let dispatcher = IPanickingDispatcher { contract_address };
    dispatcher.panic_with_byte_array();
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
