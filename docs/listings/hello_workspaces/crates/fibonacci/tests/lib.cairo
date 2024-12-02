mod abc;

#[test]
fn lib_test() {
    assert(abc::foo() == 1, '');
}
