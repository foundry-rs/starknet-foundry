use serde_json::Value;
use sierra_casm::compile;
use std::fs::File;

#[test]
fn compile_alpha6() {
    let sierra_json = read_flattened_sierra("tests/data/compilable_by_alpha6.json");

    let casm_class = compile(sierra_json);
    assert!(casm_class.is_ok());
}

#[test]
fn compile_rc0() {
    let sierra_json = read_flattened_sierra("tests/data/compilable_by_rc0.json");

    let casm_class = compile(sierra_json);
    assert!(casm_class.is_ok());
}

#[test]
fn compile_1_1_1() {
    let sierra_json = read_flattened_sierra("tests/data/compilable_by_1_1_1.json");

    let casm_class = compile(sierra_json);
    assert!(casm_class.is_ok());
}

#[test]
fn compile_latest() {
    let sierra_json = read_flattened_sierra("tests/data/compilable_by_latest.json");

    let casm_class = compile(sierra_json);
    assert!(casm_class.is_ok());
}

#[test]
fn wrong_json() {
    let sierra_json = serde_json::json!({
        "wrong": "data"
    });

    let casm_class = compile(sierra_json);
    assert!(casm_class.is_err());
}

fn read_flattened_sierra(path: &str) -> Value {
    let file = File::open(path).unwrap();
    serde_json::from_reader(file).unwrap()
}
