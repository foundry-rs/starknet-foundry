fn sum(a: felt252, b: felt252) -> felt252 {
    return a + b;
}

#[cfg(test)]
mod tests {
    use super::sum;

    #[test]
    fn test_sum() {
        assert(sum(2, 3) == 5, 'sum incorrect');
    }
}
