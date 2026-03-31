#[derive(Debug, Drop, Fuzzable)]
struct Point {
    x: u64,
    y: u64,
}

#[derive(Debug, Drop, Fuzzable)]
enum Direction {
    North,
    South,
    East,
    West,
}

// Struct with many different primitive field types
#[derive(Debug, Drop, Fuzzable)]
struct MultiField {
    a: u8,
    b: u32,
    c: u64,
    d: u128,
    e: felt252,
}

// Enum whose variants carry data
#[derive(Debug, Drop, Fuzzable)]
enum Color {
    Red: u8,
    Green: u8,
    Blue: u8,
}

// Nested structs: Outer contains Inner, both derived
#[derive(Debug, Drop, Fuzzable)]
struct Inner {
    value: u64,
}

#[derive(Debug, Drop, Fuzzable)]
struct Outer {
    inner: Inner,
    extra: u32,
}

// Struct covering fuzzable primitive types beyond the basic unsigned integers
#[derive(Debug, Drop, Fuzzable)]
struct AllPrimitives {
    flag: bool,
    signed: i64,
    address: starknet::ContractAddress,
}

#[fuzzer]
#[test]
fn test_derived_struct(p: Point) {
    let _ = p.x;
    let _ = p.y;
}

#[fuzzer]
#[test]
fn test_derived_enum(d: Direction) {
    match d {
        Direction::North => {},
        Direction::South => {},
        Direction::East => {},
        Direction::West => {},
    }
}

// All fields of MultiField are individually fuzzed
#[fuzzer]
#[test]
fn test_struct_with_multiple_field_types(m: MultiField) {
    let _ = m;
}

// The fuzzer picks a random Color variant and populates its u8 payload
#[fuzzer]
#[test]
fn test_data_enum(c: Color) {
    match c {
        Color::Red(_) => {},
        Color::Green(_) => {},
        Color::Blue(_) => {},
    }
}

// Fuzzable<Outer> recursively calls Fuzzable<Inner> for the nested field
#[fuzzer]
#[test]
fn test_nested_derived_structs(o: Outer) {
    let _ = o;
}

// Verifies Fuzzable works with bool, signed integers, and ContractAddress fields
#[fuzzer]
#[test]
fn test_struct_with_all_primitives(a: AllPrimitives) {
    let _ = a;
}

#[fuzzer(runs: 10, seed: 1)]
#[test]
fn test_failing_derived_enum(d: Direction) {
    let is_north = match d {
        Direction::North => true,
        _ => false,
    };
    // It fails deterministically since seed is set.
    assert(is_north, 'expected North');
}
