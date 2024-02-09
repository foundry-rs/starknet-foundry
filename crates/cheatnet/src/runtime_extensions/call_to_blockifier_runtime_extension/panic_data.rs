use cairo_felt::Felt252;
use conversions::byte_array::ByteArray;
use regex::Regex;

#[must_use]
pub fn try_extract_panic_data(err: &str) -> Option<Vec<Felt252>> {
    let re_felt_array = Regex::new(r"Got an exception while executing a hint: Hint Error: Execution failed\. Failure reason:\s\w*\s\('(.*)'\)\.")
        .expect("Could not create felt panic_data matching regex");

    let re_string = Regex::new(r#"(?s)Got an exception while executing a hint: Hint Error: Execution failed\. Failure reason: "(.*)"\."#)
        .expect("Could not create string panic_data matching regex");

    if let Some(captures) = re_felt_array.captures(err) {
        if let Some(panic_data_match) = captures.get(1) {
            if panic_data_match.as_str().is_empty() {
                return Some(vec![]);
            }
            let panic_data_felts: Vec<Felt252> = panic_data_match
                .as_str()
                .split(", ")
                .map(|s| Felt252::from_bytes_be(s.to_owned().as_bytes()))
                .collect();

            return Some(panic_data_felts);
        }
    }

    if let Some(captures) = re_string.captures(err) {
        if let Some(string_match) = captures.get(1) {
            let panic_data_felts: Vec<Felt252> =
                ByteArray::from(string_match.as_str().to_string()).serialize();
            return Some(panic_data_felts);
        }
    }

    None
}

#[cfg(test)]
mod test {
    use super::*;
    use cairo_felt::Felt252;

    #[test]
    fn string_extracting_panic_data() {
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
}
