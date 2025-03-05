use snforge_std::fs::{FileTrait, read_txt, read_json, FileParser};
use core::array::ArrayTrait;
use core::option::OptionTrait;
use core::serde::Serde;

fn compare_with_expected_content(content: Array<felt252>) {
    let expected = array![
        1,
        'hello',
        3,
        0x678,
        '      ',
        'hello
world',
        'world',
        0,
        3618502788666131213697322783095070105623107215331596699973092056135872020480,
    ];

    assert(content.len() == expected.len(), 'lengths not equal');
    let mut i = 0;
    while i != content.len() {
        assert(*content[i] == *expected[i], 'unexpected content');
        i += 1;
    }
}
fn compare_with_expected_content_json(content: Array<felt252>) {
    let hello: ByteArray = "hello";
    let hello_world: ByteArray = "hello
world";
    let world: ByteArray = "world";
    let spaces: ByteArray = "      ";

    let mut expected = array![1];

    hello.serialize(ref expected);

    expected.append(3);
    expected.append(0x678);

    spaces.serialize(ref expected);

    hello_world.serialize(ref expected);
    world.serialize(ref expected);

    expected.append(0);
    expected.append(3618502788666131213697322783095070105623107215331596699973092056135872020480);

    assert(content.len() == expected.len(), 'lengths not equal');

    let mut i = 0;
    while i != content.len() {
        assert(*content[i] == *expected[i], 'unexpected content');
        i += 1;
    }
}

#[derive(Serde, Drop, PartialEq)]
struct A {
    a: u32,
    nested_b: B,
    nested_d: D,
    f: felt252
}

#[derive(Serde, Drop, PartialEq)]
struct B {
    nested_c: C,
    hex: felt252,
    spaces: felt252,
    multiline: felt252,
}

#[derive(Serde, Drop, PartialEq)]
struct C {
    c: u256
}

#[derive(Serde, Drop, PartialEq)]
struct D {
    d: u64,
    e: u8
}
#[derive(Serde, Drop, PartialEq)]
struct E {
    a: felt252,
    b: F
}
#[derive(Serde, Drop, PartialEq)]
struct F {
    c: ByteArray,
    d: u8,
    e: G
}
#[derive(Serde, Drop, PartialEq)]
struct G {
    c: felt252,
}

#[derive(Serde, Destruct, Drop)]
struct Test {
    a: u8,
    array: Array<u32>,
    string_array: Array<ByteArray>,
}

#[test]
fn json_serialization() {
    let file = FileTrait::new("data/json/valid.json");
    let content = read_json(@file);
    compare_with_expected_content_json(content);
}

#[test]
fn invalid_json() {
    let file = FileTrait::new("data/json/invalid.json");
    read_json(@file);
    assert(1 == 1, '');
}

#[test]
fn json_with_array() {
    let file = FileTrait::new("data/json/with_array.json");
    let content = FileParser::<Test>::parse_json(@file).unwrap();

    let string_array = array!["test", "test2"];

    assert(*content.array[0] == 1, '1');
    assert(*content.array[1] == 23, '23');
    assert(content.string_array == string_array, 'string_array');
}

#[test]
fn json_deserialization() {
    let file = FileTrait::new("data/json/nested_valid.json");
    let content = FileParser::<E>::parse_json(@file).unwrap();

    let mut output_array = ArrayTrait::new();
    content.serialize(ref output_array);
    assert(content.a == 23, '');
    assert(content.b.c == "test", '');
    assert(content.b.e.c == 2, '');
}

#[test]
fn json_non_existent() {
    let file = FileTrait::new("data/non_existent.json");
    read_json(@file);
    assert(1 == 1, '');
}


#[test]
fn valid_content_and_same_content_no_matter_newlines() {
    let file = FileTrait::new("data/valid.txt");
    let content = FileParser::<A>::parse_txt(@file).unwrap();
    let expected = A {
        a: 1,
        nested_b: B {
            nested_c: C { c: u256 { low: 'hello', high: 3 } },
            hex: 0x678,
            spaces: '      ',
            multiline: 'hello\nworld'
        },
        nested_d: D { d: 'world', e: 0 },
        f: 3618502788666131213697322783095070105623107215331596699973092056135872020480,
    };
    assert(content.f == expected.f, '')
}

#[test]
fn serialization() {
    let file = FileTrait::new("data/valid.txt");
    let content = read_txt(@file);
    compare_with_expected_content(content);
}

#[test]
fn valid_content_different_folder() {
    let file = FileTrait::new("valid_file.txt");
    let content = read_txt(@file);
    let expected = array!['123', '12dsfwe', 124];

    assert(content.len() == expected.len(), 'lengths not equal');
    let mut i = 0;
    while i != content.len() {
        assert(*content[i] == *expected[i], 'unexpected content');
        i += 1;
    };

    assert(1 == 1, '');
}

#[test]
fn non_existent() {
    let file = FileTrait::new("data/non_existent.txt");
    read_txt(@file);
    assert(1 == 1, '');
}

#[test]
fn negative_number() {
    let file = FileTrait::new("data/negative_number.txt");
    read_txt(@file);
    assert(1 == 1, 'negative numbers not allowed');
}

#[test]
fn non_ascii() {
    let file = FileTrait::new("data/non_ascii.txt");
    read_txt(@file);
    assert(1 == 1, '');
}
