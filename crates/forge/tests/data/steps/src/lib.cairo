// 19 steps per iteration
#[cfg(test)]
mod tests {
    #[test]
    // requires 570031 steps
    fn steps_570031() {
        let mut i = 0;

        while i != 30_000 {
            i = i + 1;
            assert(1 + 1 == 2, 'who knows?');
        }
    }

    #[test]
    fn steps_5700031() {
        let mut i = 0;

        while i != 300_000 {
            i = i + 1;
            assert(1 + 1 == 2, 'who knows?');
        }
    }

    #[test]
    fn steps_3999987() {
        let mut i = 0;

        while i != 210_524 {
            i = i + 1;
            assert(1 + 1 == 2, 'who knows?');
        }
    }

    #[test]
    fn steps_4000006() {
        let mut i = 0;

        while i != 210_525 {
            i = i + 1;
            assert(1 + 1 == 2, 'who knows?');
        }
    }
}
