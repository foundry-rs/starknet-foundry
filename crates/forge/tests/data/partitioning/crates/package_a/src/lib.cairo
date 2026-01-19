#[starknet::contract]
pub mod HelloStarknet {
    #[storage]
    struct Storage {}
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_a() {
        assert!(1 + 1 == 2);
    }

    #[test]
    #[ignore] // Ignored on purpose
    fn test_b() {
        assert!(1 + 1 == 2);
    }
}
