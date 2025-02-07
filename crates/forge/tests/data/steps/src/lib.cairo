// 75/70 constant cost depending on the Cairo version
// 15 steps per iteration
#[cfg(test)]
mod tests {
    #[test]
    fn steps_much_less_than_10000000() {
        let mut i = 0;

        while i != 37_997 {
            i = i + 1;
            assert(1 + 1 == 2, 'who knows?');
        }
    }

    #[test]
    fn steps_less_than_10000000() {
        let mut i = 0;

        while i != 666_661 {
            i = i + 1;
            assert(1 + 1 == 2, 'who knows?');
        }
    }

    #[test]
    fn steps_more_than_10000000() {
        let mut i = 0;

        while i != 666_663 {
            i = i + 1;
            assert(1 + 1 == 2, 'who knows?');
        }
    }

    #[test]
    fn steps_much_more_than_10000000() {
        let mut i = 0;

        while i != 750_000 {
            i = i + 1;
            assert(1 + 1 == 2, 'who knows?');
        }
    }
}
