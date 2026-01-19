#[starknet::contract]
pub mod HelloStarknet {
    #[storage]
    struct Storage {}
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_i() {
        assert!(1 + 1 == 2);
    }

    #[test]
    fn test_j() {
        assert!(1 + 1 == 2);
    }
}
