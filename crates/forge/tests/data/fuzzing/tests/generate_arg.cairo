use snforge_std::fuzzable::generate_arg;

#[test]
fn use_generate_arg_outside_fuzzer() {
    let random: usize = generate_arg(100, 999);
    assert(99 < random && random < 1000, 'value outside correct range');
}


#[test]
fn generate_arg_incorrect_range() {
    generate_arg(101, 100);
}
