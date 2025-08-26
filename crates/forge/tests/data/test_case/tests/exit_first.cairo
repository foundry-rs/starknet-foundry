use test_case::fib;

#[test]
#[test_case(0, 1, 3)]
#[test_case(0, 1, 1000)]
fn test_fib_with_threshold(a: felt252, b: felt252, n: felt252) {
    let threshold: u256 = 10;
    let res = fib(a, b, n);
    let res: u256 = res.try_into().unwrap();
    assert!(res > threshold, "result should be greater than threshold");
}
