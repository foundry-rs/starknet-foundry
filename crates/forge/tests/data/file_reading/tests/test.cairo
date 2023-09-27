use snforge_std::io::{FileTrait, read_txt, read_json, FileParser};
use array::ArrayTrait;
use option::OptionTrait;
use serde::Serde;
use snforge_std::io::PrintTrait;

fn compare_with_expected_content(content: Array<felt252>) {
    let expected = array![
        1,
        'hello',
        3,
        'world',
        0,
        3618502788666131213697322783095070105623107215331596699973092056135872020480
    ];

    assert(content.len() == expected.len(), 'lengths not equal');
    let mut i = 0;
    loop {
        if i == content.len() {
            break;
        }
        assert(*content[i] == *expected[i], 'unexpected content');
        i += 1;
    };
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
    c: felt252,
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
    mixed_array: Array<felt252>,
    short_sting_array: Array<felt252>,
}

#[test]
fn valid_content_and_same_content_no_matter_whitespaces() {
    let file = FileTrait::new('data/valid.txt');
    let content = FileParser::<A>::parse_txt(@file).unwrap();
    let expected = A {
        a: 1, nested_b: B {
            nested_c: C { c: u256 { low: 'hello', high: 3 } }
            }, nested_d: D {
            d: 'world', e: 0
        }, f: 3618502788666131213697322783095070105623107215331596699973092056135872020480,
    };
    assert(content.f == expected.f, '')
}


#[test]
fn serialization() {
    let file = FileTrait::new('data/valid.txt');
    let content = read_txt(@file);
    compare_with_expected_content(content);

    let file = FileTrait::new('data/valid_diff_spaces.txt');
    let content = read_txt(@file);
    compare_with_expected_content(content);
}

#[test]
fn json_serialization() {
    let file = FileTrait::new('data/json/valid.json');
    let content = read_json(@file);
    compare_with_expected_content(content);
}

#[test]
#[should_panic]
fn invalid_json() {
    let file = FileTrait::new('data/json/invalid.json');
    let content = read_json(@file);
    assert(1 == 1, '');
}

#[test]
fn json_with_array() {
    let file = FileTrait::new('data/json/with_array.json');
    let content = FileParser::<Test>::parse_json(@file).unwrap();
    assert(*content.array[0] == 1, '');
    assert(*content.array[1] == 23, '');
    assert(*content.mixed_array[0] == 1, '');
    assert(*content.mixed_array[1] == 'test', '');
    assert(*content.short_sting_array[0] == 'test', '');
    assert(*content.short_sting_array[1] == 'test2', '');
}

#[test]
fn json_deserialization() {
    let file = FileTrait::new('data/json/nested_valid.json');
    let content = FileParser::<E>::parse_json(@file).unwrap();

    let mut output_array = ArrayTrait::new();
    let serialized = content.serialize(ref output_array);
    assert(content.a == 23, '');
    assert(content.b.c == 'test', '');
    assert(content.b.e.c == 2, '');
}

#[test]
fn valid_content_different_folder() {
    let file = FileTrait::new('valid_file.txt');
    let content = read_txt(@file);
    let expected = array!['123', '12dsfwe', 124];

    assert(content.len() == expected.len(), 'lengths not equal');
    let mut i = 0;
    loop {
        if i == content.len() {
            break;
        }
        assert(*content[i] == *expected[i], 'unexpected content');
        i += 1;
    };

    assert(1 == 1, '');
}

#[test]
fn non_existent() {
    let file = FileTrait::new('data/non_existent.txt');
    let content = read_txt(@file);
    assert(1 == 1, '');
}

#[test]
#[should_panic]
fn json_non_existent() {
    let file = FileTrait::new('data/non_existent.json');
    let content = read_json(@file);
    assert(1 == 1, '');
}

#[test]
fn invalid_quotes() {
    let file = FileTrait::new('data/invalid_quotes.txt');
    let content = read_txt(@file);
    assert(1 == 1, '');
}

#[test]
fn negative_number() {
    let file = FileTrait::new('data/negative_number.txt');
    let content = read_txt(@file);
    assert(1 == 1, '');
}

#[test]
fn non_ascii() {
    let file = FileTrait::new('data/non_ascii.txt');
    let content = read_txt(@file);
    assert(1 == 1, '');
}

#[test]
fn not_number_without_quotes() {
    let file = FileTrait::new('data/nan_without_quotes.txt');
    let content = read_txt(@file);
    assert(1 == 1, '');
}

#[test]
fn too_large_number() {
    let file = FileTrait::new('data/too_large_number.txt');
    let content = read_txt(@file);
    assert(1 == 1, '');
}
