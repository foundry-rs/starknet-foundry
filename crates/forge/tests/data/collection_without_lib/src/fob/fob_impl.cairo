fn fob_fn(a: felt252, b: felt252, n: felt252) -> felt252 {
    match n {
        0 => a,
        _ => fob_fn(b, a + b, n - 1),
    }
}

#[test]
fn test_fob() {
    assert(fob_fn(0, 1, 10) == 55, fob_fn(0, 1, 10));
}
