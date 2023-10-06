#[test]
#[fuzzer(test: 10)]
fn incorrect_fuzzer_arg(b: felt252) {
    assert(1 == 1, 'ok')
}

#[test]
#[fuzzer()]
fn missing_fuzzer_args(b: felt252) {
    assert(1 == 1, 'ok')
}
