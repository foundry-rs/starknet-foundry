#[starknet::contract]
pub mod HelloStarknet {
    #[storage]
    struct Storage {}
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_e() {
        assert!(1 + 1 == 2);
    }

    #[test]
    fn test_f() {
        assert!(1 + 1 == 2);
    }
}
