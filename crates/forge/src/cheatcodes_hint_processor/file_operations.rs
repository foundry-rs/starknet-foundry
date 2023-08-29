use anyhow::{anyhow, Context};
use cairo_felt::Felt252;
use cairo_lang_runner::short_string::as_cairo_short_string;
use cheatnet::cheatcodes::EnhancedHintError;
use cheatnet::cheatcodes::EnhancedHintError::FileParsing;
use flatten_serde_json::flatten;
use num_bigint::BigUint;
use serde_json::{Map, Value};

pub(super) fn read_txt(file_path: &Felt252) -> Result<Vec<Felt252>, EnhancedHintError> {
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

pub(super) fn read_json(file_path: &Felt252) -> Result<Vec<Felt252>, EnhancedHintError> {
    let file_path_str = as_cairo_short_string(file_path)
        .with_context(|| format!("Failed to convert {file_path} to str"))?;
    let content = std::fs::read_to_string(&file_path_str)?;
    let split_content = json_values_sorted_by_keys(&content).map_err(|_| FileParsing {
        path: file_path_str.clone(),
    })?;

    let felts_in_results: Vec<Result<Felt252, ()>> = split_content
        .iter()
        .map(|string| string_into_felt(string))
        .collect();

    felts_in_results
        .iter()
        .cloned()
        .collect::<Result<Vec<Felt252>, ()>>()
        .map_err(|_| FileParsing {
            path: file_path_str,
        })
}

fn json_values_sorted_by_keys(content: &str) -> Result<Vec<String>, EnhancedHintError> {
    let json: Map<String, Value> =
        serde_json::from_str(content).map_err(|_| anyhow!("Wrong format"))?;

    let data = flatten(&json);

    let mut keys: Vec<String> = data.keys().map(std::string::ToString::to_string).collect();
    keys.sort_by_key(|a| a.to_lowercase());

    Ok(keys
        .into_iter()
        .map(|key| data.get(&key).unwrap().to_string())
        .collect())
}

fn string_into_felt(string: &str) -> Result<Felt252, ()> {
    if let Ok(number) = string.parse::<BigUint>() {
        // By default it is replaced with 0 in this case
        if number < Felt252::prime() {
            Ok(number.into())
        } else {
            Err(())
        }
    } else {
        let length = string.len();
        let first_char = string.chars().next();
        let last_char = string.chars().nth(length - 1);
        dbg!(&string);
        if length >= 2
            && length - 2 <= 31
            && (first_char == Some('\'') || first_char == Some('\"'))
            && first_char == last_char
            && string.is_ascii()
        {
            let bytes = string[1..length - 1].as_bytes();
            dbg!(&bytes);
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

    use super::json_values_sorted_by_keys;

    #[test]
    fn test_json_values_sorted_by_keys() {
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
        let result = json_values_sorted_by_keys(&string).unwrap();
        let expected_result = ["1", "2", "12", "43", "\"Joh\""].to_vec();
        let result_length = expected_result.len();

        let has_proper_values = result
            .iter()
            .zip(&expected_result)
            .filter(|&(a, b)| a == b)
            .count();

        assert_eq!(has_proper_values, result_length);
    }
    #[test]
    fn test_json_values_sorted_by_keys_invalid_data() {
        let string = r#"
        [1,2,'3232']"#
            .to_owned();
        let result = json_values_sorted_by_keys(&string);
        assert!(result.is_err());

        let string = r#"
        {
            "test": "string overflowing a short string"
        }"#
        .to_owned();
        let result = json_values_sorted_by_keys(&string);
        dbg!(&result);
        assert!(result.is_err());
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
        let string2 = "\"1he5llo9\"";
        let felt = string_into_felt(string);
        let felt2 = string_into_felt(string2);
        assert_eq!(felt, felt2);
        assert_eq!(
            felt,
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
