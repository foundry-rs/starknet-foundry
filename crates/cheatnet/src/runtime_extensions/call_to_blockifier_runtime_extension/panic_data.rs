use conversions::{byte_array::ByteArray, felt::FromShortString};
use regex::Regex;
use starknet_types_core::felt::Felt;

#[must_use]
pub fn try_extract_panic_data(err: &str) -> Option<Vec<Felt>> {
    let re_felt_array = Regex::new(r"Execution failed\. Failure reason: \w+ \('(.*)'\)\.")
        .expect("Could not create felt panic_data matching regex");

    let re_string = Regex::new(r#"(?s)Execution failed\. Failure reason: "(.*?)"\."#)
        .expect("Could not create string panic_data matching regex");

    // CairoVM returns felts padded to 64 characters after 0x, unlike the spec's 63.
    // This regex (0x[a-fA-F0-9]{0,64}) handles the padded form and is different from the spec.
    let re_entry_point = Regex::new(
        r"Entry point EntryPointSelector\((0x[a-fA-F0-9]{0,64})\) not found in contract\.",
    )
    .expect("Could not create entry point panic_data matching regex");

    if let Some(captures) = re_felt_array.captures(err) {
        if let Some(panic_data_match) = captures.get(1) {
            let panic_data_felts: Vec<Felt> = panic_data_match
                .as_str()
                .split_terminator(", ")
                .map(|s| Felt::from_short_string(s).unwrap())
                .collect();

            return Some(panic_data_felts);
        }
    }

    if let Some(captures) = re_string.captures(err) {
        if let Some(string_match) = captures.get(1) {
            let string_match_str = string_match.as_str();
            let panic_data_felts: Vec<Felt> =
                ByteArray::from(string_match_str).serialize_with_magic();
            return Some(panic_data_felts);
        }
    }

    // These felts were chosen from `CairoHintProcessor` in order to be consistent with `cairo-test`:
    // https://github.com/starkware-libs/cairo/blob/2ad7718591a8d2896fec2b435c509ee5a3da9fad/crates/cairo-lang-runner/src/casm_run/mod.rs#L1055-L1057
    if let Some(_captures) = re_entry_point.captures(err) {
        let panic_data_felts = vec![
            Felt::from_bytes_be_slice("ENTRYPOINT_NOT_FOUND".as_bytes()),
            Felt::from_bytes_be_slice("ENTRYPOINT_FAILED".as_bytes()),
        ];
        return Some(panic_data_felts);
    }

    None
}

#[cfg(test)]
mod test {
    use super::*;
    use cairo_lang_utils::byte_array::BYTE_ARRAY_MAGIC;
    use conversions::{felt::FromShortString, string::TryFromHexStr};
    use indoc::indoc;
    use starknet_types_core::felt::Felt;

    #[test]
    fn extracting_plain_panic_data() {
        let cases: [(&str, Option<Vec<Felt>>); 4] = [
            (
                "Beginning of trace\nGot an exception while executing a hint: Hint Error: Execution failed. Failure reason: 0x434d3232 ('PANIK, DAYTA').\n
                 End of trace",
                Some(vec![Felt::from(344_693_033_291_u64), Felt::from(293_154_149_441_u64)])
            ),
            (
                "Got an exception while executing a hint: Hint Error: Execution failed. Failure reason: 0x434d3232 ('AYY, LMAO').",
                Some(vec![Felt::from(4_282_713_u64), Felt::from(1_280_131_407_u64)])
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
        let cases: [(&str, Option<Vec<Felt>>); 4] = [
            (
                indoc!(
                    r#"
                    Beginning of trace
                    Got an exception while executing a hint: Hint Error: Execution failed. Failure reason: "wow message is exactly 31 chars".
                    End of trace
                    "#
                ),
                Some(vec![
                    Felt::try_from_hex_str(&format!("0x{BYTE_ARRAY_MAGIC}")).unwrap(),
                    Felt::from(1),
                    Felt::from_short_string("wow message is exactly 31 chars").unwrap(),
                    Felt::from(0),
                    Felt::from(0),
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
                    Felt::try_from_hex_str(&format!("0x{BYTE_ARRAY_MAGIC}")).unwrap(),
                    Felt::from(0),
                    Felt::from(0),
                    Felt::from(0),
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
                    Felt::try_from_hex_str(&format!("0x{BYTE_ARRAY_MAGIC}")).unwrap(),
                    Felt::from(3),
                    Felt::from_short_string("A very long and multiline\nthing").unwrap(),
                    Felt::from_short_string(" is also being parsed, and can\n").unwrap(),
                    Felt::from_short_string("also can be very long as you ca").unwrap(),
                    Felt::from_short_string("n see").unwrap(),
                    Felt::from(5),
                ]),
            ),
            ("Custom Hint Error: Invalid trace: \"PANIC DATA\"", None),
        ];

        for (str, expected) in cases {
            assert_eq!(try_extract_panic_data(str), expected);
        }
    }
}
