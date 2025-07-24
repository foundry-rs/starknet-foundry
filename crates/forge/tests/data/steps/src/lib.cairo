#[cfg(test)]
mod tests {
    #[test]
    fn steps_less_than_10_000_000() {
        let mut i = 0;

        while i != 550_000 {
            i = i + 1;
            assert(1 + 1 == 2, 'who knows?');
        }
    }


    #[test]
    fn steps_more_than_10_000_000() {
        let mut i = 0;

        while i != 680_000 {
            i = i + 1;
            assert(1 + 1 == 2, 'who knows?');
        }
    }

    #[test]
    fn steps_more_than_100_000_000() {
        let mut i = 0;
        let target: felt252 = 6_800_000; 

        while i != target {
            i = i + 1;
            assert(1 + 1 == 2, 'who knows?');
        }
    }
}
