use snforge_std::declare;

#[test]
fn simple_case() {
    assert(1 == 1, 'simple check');
}

#[test]
fn contract_test() {
    declare('AdditionContract');
}
