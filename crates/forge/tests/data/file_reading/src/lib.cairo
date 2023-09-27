use snforge_std::io::{FileTrait, read_txt, FileParser};
use array::ArrayTrait;
use option::OptionTrait;
use serde::Serde;

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
