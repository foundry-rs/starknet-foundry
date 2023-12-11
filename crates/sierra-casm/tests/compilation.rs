use sierra_casm::compile;
use starknet::core::types::FlattenedSierraClass;
use std::fs::File;
use std::io::Read;

#[test]
fn compile_alpha6() {
    let flattened_sierra = read_flattened_sierra("tests/data/compilable_by_alpha6.json");

    let casm_class = compile(&flattened_sierra);
    assert!(casm_class.is_ok());
}

#[test]
fn compile_rc0() {
    let flattened_sierra = read_flattened_sierra("tests/data/compilable_by_rc0.json");

    let casm_class = compile(&flattened_sierra);
    assert!(casm_class.is_ok());
}

#[test]
fn compile_1_1_1() {
    let flattened_sierra = read_flattened_sierra("tests/data/compilable_by_1_1_1.json");

    let casm_class = compile(&flattened_sierra);
    assert!(casm_class.is_ok());
}

#[test]
fn compile_latest() {
    let flattened_sierra = read_flattened_sierra("tests/data/compilable_by_latest.json");

    let casm_class = compile(&flattened_sierra);
    assert!(casm_class.is_ok());
}

fn read_flattened_sierra(path: &str) -> FlattenedSierraClass {
    let mut file = File::open(path).unwrap();
    let mut serialized_data = String::new();
    file.read_to_string(&mut serialized_data).unwrap();

    serde_json::from_str::<FlattenedSierraClass>(&serialized_data).unwrap()
}
