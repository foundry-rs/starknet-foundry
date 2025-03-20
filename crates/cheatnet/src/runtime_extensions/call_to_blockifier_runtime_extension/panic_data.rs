use conversions::byte_array::ByteArray;
use regex::Regex;
use starknet_types_core::felt::Felt;

#[must_use]
pub fn try_extract_panic_data(err: &str) -> Option<Vec<Felt>> {
    // Regex used to extract panic data from panicking proxy contract
    let re_error_prefix = Regex::new(
        r"[\s\S]*Execution failed\. Failure reason:\nError in contract \(.+\):\n([\s\S]*).",
    )
    .expect("Could not create string panic_data matching regex");

    let re_hex = Regex::new("0x[0-9a-fA-F]+").expect("Could not create hex matching regex");

    // CairoVM returns felts padded to 64 characters after 0x, unlike the spec's 63.
    // This regex (0x[a-fA-F0-9]{0,64}) handles the padded form and is different from the spec.
    let re_entry_point = Regex::new(
        r"Entry point EntryPointSelector\((0x[a-fA-F0-9]{0,64})\) not found in contract\.",
    )
    .expect("Could not create entry point panic_data matching regex");

    if let Some(captures) = re_error_prefix.captures(err) {
        if let Some(panic_data) = captures.get(1) {
            let string_match_str = panic_data.as_str();

            // Matches panic_data when proxy contract panicked with `ByteArray`
            if string_match_str.starts_with('\"') {
                let error = string_match_str.trim_matches('"');
                let panic_data_felts: Vec<Felt> = ByteArray::from(error).serialize_with_magic();

                return Some(panic_data_felts);
            }

            // Matches `panic_data` when a proxy contract panics, either:
            // - with a single Felt "Ox"
            // - with an array of Felts "("
            // The difference comes from the `format_panic_data` implementation in `blockifier`.
            // https://github.com/starkware-libs/sequencer/blob/8211fbf1e2660884c4a9e67ddd93680495afde12/crates/starknet_api/src/execution_utils.rs
            if string_match_str.starts_with("0x") || string_match_str.starts_with('(') {
                let panic_data_felts: Vec<Felt> = re_hex
                    .find_iter(string_match_str)
                    .map(|s| Felt::from_hex(s.as_str()).unwrap())
                    .collect();
                return Some(panic_data_felts);
            }
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
                    Error at pc=0:366:
                    Got an exception while executing a hint: Execution failed. Failure reason:
                    Error in contract (contract address: 0x0033be52b9269700771b680f5905b305864f46e78bfbe79428f4bf7a933fb02f, class hash: 0x031b4bdf7360269d8bc059935f2e44d3ad487cbb781ff57527fd4b5ec13bf659, selector: 0x032e90fe8c4355e4732f08747d73146ef03dcd019ec3498c089dce91cf40aadc):
                    0x1.
                    "
                ),
                    Some(&vec![Felt::from(1)]); "non ascii felt")]
    #[test_case(indoc!(r"
                    Error at pc=0:366:
                    Got an exception while executing a hint: Execution failed. Failure reason:
                    Error in contract (contract address: 0x0033be52b9269700771b680f5905b305864f46e78bfbe79428f4bf7a933fb02f, class hash: 0x031b4bdf7360269d8bc059935f2e44d3ad487cbb781ff57527fd4b5ec13bf659, selector: 0x032e90fe8c4355e4732f08747d73146ef03dcd019ec3498c089dce91cf40aadc):
                    0x41 ('A').
                    "
                ),
                    Some(&vec![Felt::from(65)]); "ascii felt")]
    #[test_case(indoc!(r"
                    Error at pc=0:366:
                    Got an exception while executing a hint: Execution failed. Failure reason:
                    Error in contract (contract address: 0x0033be52b9269700771b680f5905b305864f46e78bfbe79428f4bf7a933fb02f, class hash: 0x031b4bdf7360269d8bc059935f2e44d3ad487cbb781ff57527fd4b5ec13bf659, selector: 0x032e90fe8c4355e4732f08747d73146ef03dcd019ec3498c089dce91cf40aadc):
                    (0x1, 0x41 ('A'), 0x2, 0x42 ('B')).
                    "
                ),
                    Some(&vec![Felt::from(1), Felt::from(65), Felt::from(2), Felt::from(66)]); "mixed felts")]
    #[test_case(indoc!(r"
                    Got an exception while executing a hint: Execution failed. Failure reason:
                    Error in contract (contract address: 0x03cda836debfed3f83aa981d7a31733da3ae4f903dde9d833509d2f985d52241, class hash: 0x07ca8b953cb041ee517951d34880631e537682103870b9b018a7b493363b9b63, selector: 0x00a4695e9e8c278609a8e9362d5abe9852a904da970c7de84f0456c777d21137):
                    (0x54687265652073616420746967657273206174652077686561742e2054776f ('Three sad tigers ate wheat. Two'), 0x2074696765727320776572652066756c6c2e20546865206f74686572207469 (' tigers were full. The other ti'),
                    0x676572206e6f7420736f206d756368 ('ger not so much')).
                    "
                ),
                    Some(&vec![Felt::from_hex_unchecked("0x54687265652073616420746967657273206174652077686561742e2054776f"), Felt::from_hex_unchecked("0x2074696765727320776572652066756c6c2e20546865206f74686572207469"), Felt::from_hex_unchecked("0x676572206e6f7420736f206d756368")]); "felt array")]
    fn extracting_plain_panic_data(data: &str, expected: Option<&Vec<Felt>>) {
        assert_eq!(try_extract_panic_data(data), expected.cloned());
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
