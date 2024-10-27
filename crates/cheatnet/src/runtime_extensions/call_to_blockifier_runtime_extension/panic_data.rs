use cairo_vm::Felt252;
use conversions::{byte_array::ByteArray, felt252::FromShortString};
use regex::Regex;

#[must_use]
pub fn try_extract_panic_data(err: &str) -> Option<Vec<Felt252>> {
    let re_felt_array = Regex::new(r"Execution failed\. Failure reason: \w+ \('(.*)'\)\.")
        .expect("Could not create felt panic_data matching regex");

    let re_string = Regex::new(r#"(?s)Execution failed\. Failure reason: "(.*?)"\."#)
        .expect("Could not create string panic_data matching regex");

    let re_entry_point =
        Regex::new(r"Entry point EntryPointSelector\((0x[0-9a-fA-F]+)\) not found in contract\.")
            .unwrap();

    if let Some(captures) = re_felt_array.captures(err) {
        if let Some(panic_data_match) = captures.get(1) {
            let panic_data_felts: Vec<Felt252> = panic_data_match
                .as_str()
                .split_terminator(", ")
                .map(|s| Felt252::from_short_string(s).unwrap())
                .collect();

            return Some(panic_data_felts);
        }
    }

    if let Some(captures) = re_string.captures(err) {
        if let Some(string_match) = captures.get(1) {
            let string_match_str = string_match.as_str();
            let panic_data_felts: Vec<Felt252> =
                ByteArray::from(string_match_str).serialize_with_magic();
            return Some(panic_data_felts);
        }
    }

    if let Some(_captures) = re_entry_point.captures(err) {
        let panic_data_felts = vec![
            Felt252::from_bytes_be_slice("ENTRYPOINT_NOT_FOUND".as_bytes()),
            Felt252::from_bytes_be_slice("ENTRYPOINT_FAILED".as_bytes()),
        ];
        return Some(panic_data_felts);
    }

    None
}

#[cfg(test)]
mod test {
    use super::*;
    use cairo_lang_utils::byte_array::BYTE_ARRAY_MAGIC;
    use cairo_vm::Felt252;
    use conversions::{felt252::FromShortString, string::TryFromHexStr};
    use indoc::indoc;

    #[test]
    fn extracting_plain_panic_data() {
        let cases: [(&str, Option<Vec<Felt252>>); 4] = [
            (
                "Beginning of trace\nGot an exception while executing a hint: Hint Error: Execution failed. Failure reason: 0x434d3232 ('PANIK, DAYTA').\n
                 End of trace",
                Some(vec![Felt252::from(344_693_033_291_u64), Felt252::from(293_154_149_441_u64)])
            ),
            (
                "Got an exception while executing a hint: Hint Error: Execution failed. Failure reason: 0x434d3232 ('AYY, LMAO').",
                Some(vec![Felt252::from(4_282_713_u64), Felt252::from(1_280_131_407_u64)])
            ),
            (
                "Got an exception while executing a hint: Hint Error: Execution failed. Failure reason: 0x0 ('').",
                Some(vec![])
            ),
            ("Custom Hint Error: Invalid trace: \"PANIC, DATA\"", None)
        ];

        for (str, expected) in cases {
            assert_eq!(try_extract_panic_data(str), expected);
        }
    }

    #[test]
    fn extracting_string_panic_data() {
        let cases: [(&str, Option<Vec<Felt252>>); 4] = [
            (
                indoc!(
                    r#"
                    Beginning of trace
                    Got an exception while executing a hint: Hint Error: Execution failed. Failure reason: "wow message is exactly 31 chars".
                    End of trace
                    "#
                ),
                Some(vec![
                    Felt252::try_from_hex_str(&format!("0x{BYTE_ARRAY_MAGIC}")).unwrap(),
                    Felt252::from(1),
                    Felt252::from_short_string("wow message is exactly 31 chars").unwrap(),
                    Felt252::from(0),
                    Felt252::from(0),
                ]),
            ),
            (
                indoc!(
                    r#"
                    Beginning of trace
                    Got an exception while executing a hint: Hint Error: Execution failed. Failure reason: "".
                    End of trace
                    "#
                ),
                Some(vec![
                    Felt252::try_from_hex_str(&format!("0x{BYTE_ARRAY_MAGIC}")).unwrap(),
                    Felt252::from(0),
                    Felt252::from(0),
                    Felt252::from(0),
                ]),
            ),
            (
                indoc!(
                    r#"
                    Beginning of trace
                    Got an exception while executing a hint: Hint Error: Execution failed. Failure reason: "A very long and multiline
                    thing is also being parsed, and can
                    also can be very long as you can see".
                    End of trace
                    "#
                ),
                Some(vec![
                    Felt252::try_from_hex_str(&format!("0x{BYTE_ARRAY_MAGIC}")).unwrap(),
                    Felt252::from(3),
                    Felt252::from_short_string("A very long and multiline\nthing").unwrap(),
                    Felt252::from_short_string(" is also being parsed, and can\n").unwrap(),
                    Felt252::from_short_string("also can be very long as you ca").unwrap(),
                    Felt252::from_short_string("n see").unwrap(),
                    Felt252::from(5),
                ]),
            ),
            ("Custom Hint Error: Invalid trace: \"PANIC DATA\"", None),
        ];

        for (str, expected) in cases {
            assert_eq!(try_extract_panic_data(str), expected);
        }
    }
}
