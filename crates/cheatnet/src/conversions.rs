use cairo_felt::Felt252;
use starknet::core::utils::get_selector_from_name;

#[must_use]
pub fn felt_selector_from_name(name: &str) -> Felt252 {
    let selector = get_selector_from_name(name).unwrap();
    Felt252::from_bytes_be(&selector.to_bytes_be())
}

#[must_use]
pub fn felt_from_short_string(short_str: &str) -> Felt252 {
    return Felt252::from_bytes_be(short_str.as_bytes());
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parsing_felt_from_short_string() {
        let cases = [
            ("", Felt252::from(0)),
            ("{", Felt252::from(123)),
            ("PANIK", Felt252::from(344_693_033_291_u64)),
        ];

        for (str, felt_res) in cases {
            assert_eq!(felt_from_short_string(str), felt_res);
        }
    }
}
