fn sum(a: felt252, b: felt252) -> felt252 {
    return a + b;
}

#[cfg(test)]
mod tests {
    use super::sum;

    #[test]
    fn test_sum(x: felt252, y: felt252) {
        assert_eq!(sum(x, y), x + y);
    }
}
