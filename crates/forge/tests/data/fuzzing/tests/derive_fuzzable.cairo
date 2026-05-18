#[derive(Debug, Drop, Fuzzable)]
struct InnerStruct {
    value: u64,
}

#[derive(Debug, Drop, Fuzzable)]
struct OuterStruct {
    inner: InnerStruct,
    extra: u32,
}

#[derive(Debug, Drop, Fuzzable)]
struct MultiField<T> {
    a: u256,
    b: u128,
    c: felt252,
    flag: bool,
    signed: i64,
    address: starknet::ContractAddress,
    t: T,
}

#[derive(Debug, Drop, Fuzzable)]
enum InnerEnum {
    Red: u8,
    Green,
    Blue: u32,
}

#[derive(Debug, Drop, Fuzzable)]
enum OuterEnum<T> {
    North: u8,
    South,
    East: InnerEnum,
    West: OuterStruct,
    Other: MultiField<T>,
}

#[fuzzer(runs: 10, seed: 1)]
#[test]
fn test_derived_enum(d: OuterEnum<u8>) {
    println!("{d:?}");
    let is_north = match d {
        OuterEnum::North(_) => true,
        _ => false,
    };
    // It fails deterministically since seed is set.
    assert(is_north, 'expected North');
}
