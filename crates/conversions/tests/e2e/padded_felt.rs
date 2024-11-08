#[cfg(test)]
mod tests {
    use conversions::padded_felt::PaddedFelt;
    use conversions::{FromConv, IntoConv};
    use starknet_api::core::{ClassHash, ContractAddress};
    use starknet_types_core::felt::Felt;

    #[test]
    fn test_padded_felt_conversions_happy_case() {
        let felt = Felt::from_bytes_be(&[1u8; 32]);
        let padded_felt = PaddedFelt(felt);

        assert_eq!(padded_felt, ContractAddress::from_(padded_felt).into_());
        assert_eq!(padded_felt, ClassHash::from_(padded_felt).into_());
    }
}
