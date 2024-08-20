// 75 constant cost
// 15 steps per iteration
#[cfg(test)]
mod tests {
    #[test]
    // requires 570030 steps
    fn steps_570030() {
        let mut i = 0;

        while i != 37_997 {
            i = i + 1;
            assert(1 + 1 == 2, 'who knows?');
        }
    }

    #[test]
    fn steps_9999990() {
        let mut i = 0;

        while i != 666_661 {
            i = i + 1;
            assert(1 + 1 == 2, 'who knows?');
        }
    }

    #[test]
    fn steps_10000005() {
        let mut i = 0;

        while i != 666_662 {
            i = i + 1;
            assert(1 + 1 == 2, 'who knows?');
        }
    }

    #[test]
    fn steps_11250075() {
        let mut i = 0;

        while i != 750_000 {
            i = i + 1;
            assert(1 + 1 == 2, 'who knows?');
        }
    }
}
