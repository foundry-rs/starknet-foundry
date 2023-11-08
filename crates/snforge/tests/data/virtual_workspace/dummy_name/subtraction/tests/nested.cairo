use snforge_std::declare;

mod test_nested;

fn foo() -> u8 {
    2
}

#[test]
fn simple_case() {
    assert(1 == 1, 'simple check');
}

#[test]
fn contract_test() {
    declare('SubtractionContract');
}
