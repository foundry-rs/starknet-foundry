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
    fn steps_5699625() {
        let mut i = 0;

        while i != 379_970 {
            i = i + 1;
            assert(1 + 1 == 2, 'who knows?');
        }
    }

    #[test]
    fn steps_3999990() {
        let mut i = 0;

        while i != 266_661 {
            i = i + 1;
            assert(1 + 1 == 2, 'who knows?');
        }
    }

    #[test]
    fn steps_4000005() {
        let mut i = 0;

        while i != 266_662 {
            i = i + 1;
            assert(1 + 1 == 2, 'who knows?');
        }
    }
}
