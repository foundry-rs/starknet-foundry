use num_bigint::BigInt;
use std::str;

use crate::decompiler::libfuncs_patterns::TYPE_ID_REGEX;
use crate::decompiler::libfuncs_patterns::USER_DEFINED_TYPE_ID_REGEX;

/// Convert an integer to it's string value or hex value
/// Used to decode consts
#[inline]
pub fn decode_hex_bigint(bigint: &BigInt) -> Option<String> {
    // Convert the BigInt to a hexadecimal string
    let hex_string = format!("{:x}", bigint);

    // Decode the hexadecimal string to a byte vector
    let bytes = hex::decode(hex_string.clone()).ok()?;

    // Convert the byte vector to a string or hex value
    let string = match str::from_utf8(&bytes) {
        Ok(s) => Some(s.to_string()),
        Err(_) => Some(format!("0x{hex_string}")),
    };

    string
}

/// Replaces type IDs in the given invocation string with the corresponding type names from the declared_types_names list
/// If there are no matches or if there is an error in the process, the original string is returned
pub fn replace_types_id(declared_types_names: &Vec<String>, invocation: &str) -> String {
    // Use the TYPE_ID_REGEX to replace all matches in the invocation string
    TYPE_ID_REGEX
        .replace_all(&invocation, |caps: &regex::Captures| {
            // Get the type ID from the capture group
            caps.name("type_id")
                // Parse the type ID as a usize, if possible
                .and_then(|type_id| {
                    let type_id_str = type_id.as_str();
                    // Check if the type ID is not preceded by "user@" (to avoid mistakes w/ user defined functions)
                    if !caps
                        .get(0)
                        .unwrap()
                        .start()
                        .checked_sub(5)
                        .map_or(false, |i| &invocation[i..i + 5] == "user@")
                    {
                        // If the type ID is not preceded by "user@", parse it as a usize
                        type_id_str
                            .trim_matches(|c| c == '[' || c == ']')
                            .parse::<usize>()
                            .ok()
                    } else {
                        // If the type ID is preceded by "user@", return None
                        None
                    }
                })
                // Use the parsed type ID as an index into the declared_types_names list
                .and_then(|index| declared_types_names.get(index).cloned())
                // If there was an error, return the original type ID
                .unwrap_or_else(|| caps[0].to_string())
        })
        // Convert the result to a string
        .to_string()
}

/// "Decode" (simplify) a user-defined type ID by truncating it to the 4th character
/// or return the type_id if it does not match the USER_DEFINED_TYPE_ID_REGEX regex pattern
pub fn decode_user_defined_type_id(type_id: String) -> String {
    if let Some(captures) = USER_DEFINED_TYPE_ID_REGEX.captures(&type_id) {
        // If the type ID matches the regex pattern, truncate it to the 4th character
        if let Some(type_id_match) = captures.name("type_id") {
            let truncated_type_id = &type_id_match.as_str()[..4];
            format!("ut@[{}...]", truncated_type_id)
        } else {
            // If the type ID does not match the regex pattern, return the original input string
            type_id
        }
    } else {
        // If the input string does not match the regex pattern, return the original input string
        type_id
    }
}
