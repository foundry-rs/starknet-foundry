use conversions::{byte_array::ByteArray, felt::FromShortString};
use regex::Regex;
use starknet_types_core::felt::Felt;

#[must_use]
pub fn try_extract_panic_data(err: &str) -> Option<Vec<Felt>> {
    let re_felt_array = Regex::new(
        r"[\s\S]*Execution failed\. Failure reason:\nError in contract \(.+\):\n.*\('([\s\S]*)'\).",
    )
    .expect("Could not create felt panic_data matching regex");

    let re_string = Regex::new(
        r#"[\s\S]*Execution failed\. Failure reason:\nError in contract \(.+\):\n"([\s\S]*)"."#,
    )
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
    use test_case::test_case;

    #[test_case(indoc!(r"
                    Beginning of trace
                    Got an exception while executing a hint: Execution failed. Failure reason:
                    Error in contract (contract address: 0x03cda836debfed3f83aa981d7a31733da3ae4f903dde9d833509d2f985d52241, class hash: 0x07ca8b953cb041ee517951d34880631e537682103870b9b018a7b493363b9b63, selector: 0x00a4695e9e8c278609a8e9362d5abe9852a904da970c7de84f0456c777d21137):
                    0x434d3232 ('PANIK, DAYTA').
                    End of trace
                    "
                ),
                    Some(vec![Felt::from(344_693_033_291_u64), Felt::from(293_154_149_441_u64)]); "two felts")]
    #[test_case(indoc!(r"
                    Got an exception while executing a hint: Execution failed. Failure reason:
                    Error in contract (contract address: 0x03cda836debfed3f83aa981d7a31733da3ae4f903dde9d833509d2f985d52241, class hash: 0x07ca8b953cb041ee517951d34880631e537682103870b9b018a7b493363b9b63, selector: 0x00a4695e9e8c278609a8e9362d5abe9852a904da970c7de84f0456c777d21137):
                    0x0 ('').
                    "
                ),
                    Some(vec![]); "empty")]
    fn extracting_plain_panic_data(data: &str, expected: Option<Vec<Felt>>) {
        assert_eq!(try_extract_panic_data(data), expected);
    }

    #[allow(clippy::needless_pass_by_value)]
    #[test_case(indoc!(
                    r#"
                    Error at pc=0:107:
                    Got an exception while executing a hint: Execution failed. Failure reason:
                    Error in contract (contract address: 0x03cda836debfed3f83aa981d7a31733da3ae4f903dde9d833509d2f985d52241, class hash: 0x07ca8b953cb041ee517951d34880631e537682103870b9b018a7b493363b9b63, selector: 0x00a4695e9e8c278609a8e9362d5abe9852a904da970c7de84f0456c777d21137):
                    "wow message is exactly 31 chars".
                    "#
                ), Some(vec![
                    Felt::try_from_hex_str(&format!("0x{BYTE_ARRAY_MAGIC}")).unwrap(),
                    Felt::from(1),
                    Felt::from_short_string("wow message is exactly 31 chars").unwrap(),
                    Felt::from(0),
                    Felt::from(0),
                ]);
                "exactly 31 chars"
    )]
    #[test_case(indoc!(
                    r#"
                    Error at pc=0:107:
                    Got an exception while executing a hint: Execution failed. Failure reason:
                    Error in contract (contract address: 0x03cda836debfed3f83aa981d7a31733da3ae4f903dde9d833509d2f985d52241, class hash: 0x07ca8b953cb041ee517951d34880631e537682103870b9b018a7b493363b9b63, selector: 0x00a4695e9e8c278609a8e9362d5abe9852a904da970c7de84f0456c777d21137):
                    "".
                    "#
                ),
                Some(vec![
                    Felt::try_from_hex_str(&format!("0x{BYTE_ARRAY_MAGIC}")).unwrap(),
                    Felt::from(0),
                    Felt::from(0),
                    Felt::from(0),
                ]);
                "empty string"
    )]
    #[test_case(indoc!(
                    r#"
                    Error at pc=0:107:
                    Got an exception while executing a hint: Execution failed. Failure reason:
                    Error in contract (contract address: 0x03cda836debfed3f83aa981d7a31733da3ae4f903dde9d833509d2f985d52241, class hash: 0x07ca8b953cb041ee517951d34880631e537682103870b9b018a7b493363b9b63, selector: 0x00a4695e9e8c278609a8e9362d5abe9852a904da970c7de84f0456c777d21137):
                    "A very long and multiline
                    thing is also being parsed, and can
                    also can be very long as you can see".
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
                ]);
                "long string"
    )]
    #[test_case("Custom Hint Error: Invalid trace: \"PANIC DATA\"", None; "invalid")]
    fn extracting_string_panic_data(data: &str, expected: Option<Vec<Felt>>) {
        assert_eq!(try_extract_panic_data(data), expected);
    }
}
