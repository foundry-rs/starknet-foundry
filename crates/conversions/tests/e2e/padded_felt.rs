#[cfg(test)]
mod tests {
    use conversions::padded_felt::PaddedFelt;
    use conversions::{FromConv, IntoConv};
    use starknet_api::core::{ClassHash, ContractAddress};
    use starknet_types_core::felt::Felt;

    #[test]
    fn test_padded_felt_lower_hex() {
        let felt = Felt::from_hex("0x123").unwrap();
        let padded_felt = PaddedFelt(felt);

        assert_eq!(
            format!("{:x}", padded_felt),
            "0x0000000000000000000000000000000000000000000000000000000000000123".to_string()
        );
    }

    #[test]
    fn test_padded_felt_conversions_happy_case() {
        let felt = Felt::from_bytes_be(&[1u8; 32]);
        let padded_felt = PaddedFelt(felt);

        assert_eq!(padded_felt, ContractAddress::from_(padded_felt).into_());
        assert_eq!(padded_felt, ClassHash::from_(padded_felt).into_());
    }

    #[test]
    fn test_padded_felt_serialize() {
        let felt = Felt::ONE;
        let padded_felt = PaddedFelt(felt);

        let serialized = serde_json::to_string(&padded_felt).unwrap();
        assert_eq!(
            serialized,
            "\"0x0000000000000000000000000000000000000000000000000000000000000001\"".to_string()
        );
    }
}
