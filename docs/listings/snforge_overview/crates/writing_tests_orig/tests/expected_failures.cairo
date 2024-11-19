//ANCHOR:byte_array
#[test]
#[should_panic(expected: "This will panic")]
fn should_panic_exact() {
    panic!("This will panic");
}

// here the expected message is a substring of the actual message
#[test]
#[should_panic(expected: "will panic")]
fn should_panic_expected_is_substring() {
    panic!("This will panic");
}
//ANCHOR_END:byte_array

//ANCHOR:felt
#[test]
#[should_panic(expected: 'panic message')]
fn should_panic_felt_matching() {
    assert(1 != 1, 'panic message');
}
//ANCHOR_END:felt

//ANCHOR:tuple
use core::panic_with_felt252;

#[test]
#[should_panic(expected: ('panic message',))]
fn should_panic_check_data() {
    panic_with_felt252('panic message');
}

// works for multiple messages
#[test]
#[should_panic(expected: ('panic message', 'second message',))]
fn should_panic_multiple_messages() {
    let mut arr = ArrayTrait::new();
    arr.append('panic message');
    arr.append('second message');
    panic(arr);
}
//ANCHOR_END:tuple

mod dummy {} // trick `scarb fmt -c`
