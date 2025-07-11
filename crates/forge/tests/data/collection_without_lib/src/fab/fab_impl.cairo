use collection_without_lib::fib::fib_fn;

pub fn fab_fn(a: felt252, b: felt252, n: felt252) -> felt252 {
    match n {
        0 => a,
        _ => fab_fn(b, a + b, n - 1),
    }
}

#[cfg(test)]
mod tests {
    use super::{fab_fn, fib_fn, fn_from_above};

    #[test]
    fn test_fab() {
        assert(fab_fn(0, 1, 10) == 55, fab_fn(0, 1, 10));
    }

    #[test]
    fn test_how_does_this_work() {
        assert(fib_fn(0, 1, 10) == 55, fib_fn(0, 1, 10));
    }

    #[test]
    fn test_super() {
        let one: felt252 = 1;
        assert(fn_from_above() == one, 1);
    }
}
