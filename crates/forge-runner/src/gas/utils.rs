use starknet_types_core::felt::Felt;

pub(super) fn shorten_felt(felt: Felt) -> String {
    let padded = format!("{felt:#066x}");
    let first = &padded[..4];
    let last = &padded[padded.len() - 4..];
    format!("{first}…{last}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_long() {
        let felt = Felt::from_hex_unchecked(
            "0x01c902da594beda43db10142ecf1fc3a098b56e8d95f3cd28587a0c6ba05a451",
        );
        assert_eq!("0x01…a451", &shorten_felt(felt));
    }

    #[test]
    fn test_short() {
        let felt = Felt::from_hex_unchecked("0x123");
        assert_eq!("0x00…0123", &shorten_felt(felt));
    }
}
