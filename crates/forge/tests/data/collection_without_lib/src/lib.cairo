pub mod fab;
pub mod fib;
pub mod fob;
use fib::fib_fn;
use fob::fob_impl::fob_fn;

#[cfg(test)]
mod tests {
    use super::{fib_fn, fob_fn};
    #[test]
    fn test_simple() {
        assert(1 == 1, 1);
    }

    #[test]
    fn test_fob_in_lib() {
        assert(fob_fn(0, 1, 10) == 55, fob_fn(0, 1, 10));
    }

    #[test]
    fn test_fib_in_lib() {
        assert(fib_fn(0, 1, 10) == 55, fib_fn(0, 1, 10));
    }
}
