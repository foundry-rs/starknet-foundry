#[cfg(test)]
mod tests {
    #[test]
    // requires 570031 steps
    fn steps_570031() {
        let mut i = 0;

        loop {
            if i == 30_000 {
                break;
            }

            i = i + 1;

            assert(1 + 1 == 2, 'who knows?');
        }
    }

    #[test]
    fn steps_5700031() {
        let mut i = 0;

        loop {
            if i == 300_000 {
                break;
            }

            i = i + 1;

            assert(1 + 1 == 2, 'who knows?');
        }
    }

    #[test]
    fn steps_2999998() {
        let mut i = 0;

        loop {
            if i == 157_893 {
                break;
            }

            i = i + 1;

            assert(1 + 1 == 2, 'who knows?');
        }
    }

    #[test]
    fn steps_3000017() {
        let mut i = 0;

        loop {
            if i == 157_894 {
                break;
            }

            i = i + 1;

            assert(1 + 1 == 2, 'who knows?');
        }
    }
}
