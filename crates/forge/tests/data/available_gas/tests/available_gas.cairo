use array::ArrayTrait;

#[test]
fn aa_test() {
    assert(1 == 1, 'ok')
}

#[test]
#[available_gas(10000)]
fn available_gas() {
    assert(1 == 1, 'ok')
}

#[test]
fn test() {
    assert(1 == 1, 'ok')
}

