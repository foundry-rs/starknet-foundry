#[test]
#[fuzzer]
fn fuzzer_after_test() {
    let _x = 10;
}

#[fuzzer]
#[test]
fn fuzzer_before_test() {
    let _x = 10;
}
