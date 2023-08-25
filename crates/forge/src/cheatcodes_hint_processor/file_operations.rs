use anyhow::Context;
use cairo_felt::Felt252;
use cairo_lang_runner::short_string::as_cairo_short_string;
use cheatnet::cheatcodes::EnhancedHintError;
use cheatnet::cheatcodes::EnhancedHintError::FileParsing;
use flatten_serde_json::flatten;
use num_bigint::BigUint;
use serde_json::{Map, Value};

pub(super) fn parse_txt(file_path: &Felt252) -> Result<Vec<Felt252>, EnhancedHintError> {
    let file_path_str = as_cairo_short_string(file_path)
        .with_context(|| format!("Failed to convert {file_path} to str"))?;
    let content = std::fs::read_to_string(file_path_str.clone())?;
    let split_content: Vec<&str> = content.trim().split_ascii_whitespace().collect();

    let felts_in_results: Vec<Result<Felt252, ()>> = split_content
        .iter()
        .map(|&string| string_into_felt(string))
        .collect();

    felts_in_results
        .iter()
        .cloned()
        .collect::<Result<Vec<Felt252>, ()>>()
        .map_err(|_| FileParsing {
            path: file_path_str,
        })
}

pub(super) fn parse_json(file_path: &Felt252) -> Result<Vec<Felt252>, EnhancedHintError> {
    let file_path_str = as_cairo_short_string(file_path)
        .with_context(|| format!("Failed to convert {file_path} to str"))?;
    let content = std::fs::read_to_string(file_path_str.clone())?;
    let split_content = json_to_alphanumeric_sorted_vec(content);

    // let split_content: Vec<&str> = content.trim().split_ascii_whitespace().collect();
    let felts_in_results: Vec<Result<Felt252, ()>> = split_content
        .iter()
        .map(|string| string_into_felt(&string))
        .collect();

    felts_in_results
        .iter()
        .cloned()
        .collect::<Result<Vec<Felt252>, ()>>()
        .map_err(|_| FileParsing {
            path: file_path_str,
        })
}

fn json_to_alphanumeric_sorted_vec(content: String) -> Vec<String> {
    let json: Map<String, Value> = serde_json::from_str(&content).unwrap();
    let data = flatten(&json);

    let mut keys: Vec<String> = data.keys().map(|e| e.to_string()).collect();
    keys.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

    keys.into_iter()
        .map(|key| data.get(&key).unwrap().to_string().replace("\"", "\'"))
        .collect()
}

fn string_into_felt(string: &str) -> Result<Felt252, ()> {
    if let Ok(number) = string.parse::<BigUint>() {
        // By default it is replaced with 0 in this cases
        if number < Felt252::prime() {
            Ok(number.into())
        } else {
            Err(())
        }
    } else {
        let length = string.len();
        let first_char = string.chars().next();
        let last_char = string.chars().nth(length - 1);

        if length >= 2
            && length - 2 <= 31
            && first_char == Some('\'')
            && first_char == last_char
            && string.is_ascii()
        {
            let bytes = string[1..length - 1].as_bytes();
            Ok(Felt252::from_bytes_be(bytes))
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cheatcodes_hint_processor::file_operations::string_into_felt;
    use cairo_felt::Felt252;
    use num_bigint::BigUint;
    use num_traits::One;

    use super::json_to_alphanumeric_sorted_vec;

    #[test]
    fn test_parse_json() {
        let string = r#"
        {
            "name": "Joh",
            "age": 43,
            "a": {
                "b": 1,
                "c": 2
            },
            "ab": 12
        }"#
        .to_owned();
        let result = json_to_alphanumeric_sorted_vec(string);
        let expected_result = ["1", "2", "12", "43", "Joh"].to_vec();
        let result_length = expected_result.len();

        let has_proper_values = result
            .iter()
            .zip(&expected_result)
            .filter(|&(a, b)| a == b)
            .count();

        assert_eq!(has_proper_values, result_length)
    }

    #[test]
    fn test_string_into_felt_positive() {
        let string = "123";
        assert_eq!(string_into_felt(string), Ok(123.into()));
    }

    #[test]
    fn test_string_into_felt_negative() {
        let string = "-123";
        assert_eq!(string_into_felt(string), Err(()));
    }

    #[test]
    fn test_string_into_felt_prime_minus_one() {
        let string = "3618502788666131213697322783095070105623107215331596699973092056135872020480";
        let expected = Felt252::prime() - BigUint::one();
        assert_eq!(string_into_felt(string), Ok(expected.into()));
    }

    #[test]
    fn test_string_into_felt_prime() {
        let string = "3618502788666131213697322783095070105623107215331596699973092056135872020481";
        assert_eq!(string_into_felt(string), Err(()));
    }

    #[test]
    fn test_string_into_felt_nan() {
        let string = "A2bA";
        assert_eq!(string_into_felt(string), Err(()));
    }

    #[test]
    fn test_string_into_felt_shortstring_single_quotes() {
        let string = "\'1he5llo9\'";
        assert_eq!(
            string_into_felt(string),
            Ok(Felt252::from_bytes_be(
                string[1..string.len() - 1].as_bytes()
            ))
        );
    }

    #[test]
    fn test_string_into_felt_shortstring_missmatched_quotes() {
        let string = "\'1he5llo9\"";
        assert_eq!(string_into_felt(string), Err(()));
        let string = "\"1he5llo9\'";
        assert_eq!(string_into_felt(string), Err(()));
    }

    #[test]
    fn test_string_into_felt_shortstring_missing_quote() {
        let string = "\'1he5llo9";
        assert_eq!(string_into_felt(string), Err(()));
    }

    #[test]
    fn test_string_into_felt_shortstring_empty() {
        let string = "\'\'";
        assert_eq!(string_into_felt(string), Ok(0.into()));
    }

    #[test]
    fn test_string_into_felt_shortstring_too_long() {
        let string = "\'abcdefghjiklmnoprstqwyzabcdefghi\'";
        assert_eq!(string_into_felt(string), Err(()));
    }

    #[test]
    fn test_string_into_felt_shortstring_non_ascii() {
        let string = "\'abcd§g\'";
        assert_eq!(string_into_felt(string), Err(()));
    }
}
