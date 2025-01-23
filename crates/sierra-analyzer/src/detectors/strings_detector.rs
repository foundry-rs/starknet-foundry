use crate::decompiler::decompiler::Decompiler;
use crate::decompiler::libfuncs_patterns::CONST_REGEXES;
use crate::decompiler::utils::decode_hex_bigint;
use crate::decompiler::utils::replace_types_id;
use crate::detectors::detector::{Detector, DetectorType};
use crate::parse_element_name;

use cairo_lang_sierra::program::GenStatement;
use num_bigint::BigInt;
use std::collections::BTreeSet;

#[derive(Debug)]
pub struct StringsDetector;

impl StringsDetector {
    /// Creates a new `StringsDetector` instance
    pub fn new() -> Self {
        Self
    }
}

impl Detector for StringsDetector {
    /// Returns the id of the detector
    #[inline]
    fn id(&self) -> &'static str {
        "strings"
    }

    /// Returns the name of the detector
    #[inline]
    fn name(&self) -> &'static str {
        "Strings"
    }

    /// Returns the description of the detector
    #[inline]
    fn description(&self) -> &'static str {
        "Detects strings in the decompiled Sierra code."
    }

    /// Returns the type of the detector
    #[inline]
    fn detector_type(&self) -> DetectorType {
        DetectorType::INFORMATIONAL
    }

    /// Detects unique strings in the decompiled Sierra code and returns them as a single string
    fn detect(&mut self, decompiler: &mut Decompiler) -> String {
        // A set to store the extracted unique strings
        // We use a BTreeSet instead of HashSet to get an ordered result
        let mut extracted_strings: BTreeSet<String> = BTreeSet::new();

        // Iterate over all the program statements
        for function in &decompiler.functions {
            for statement in &function.statements {
                let statement = &statement.statement;
                match statement {
                    GenStatement::Invocation(invocation) => {
                        // Parse the ID of the invoked library function
                        let libfunc_id_str = parse_element_name!(invocation.libfunc_id);

                        // If the libfunc id is an integer
                        let libfunc_id_str = if let Ok(index) = libfunc_id_str.parse::<usize>() {
                            // If it's a remote contract we try to convert the types IDs to their equivalents types names
                            if let Some(libfunc_name) =
                                decompiler.declared_libfuncs_names.get(index)
                            {
                                replace_types_id(&decompiler.declared_types_names, libfunc_name)
                            } else {
                                continue;
                            }
                        } else {
                            parse_element_name!(invocation.libfunc_id)
                        };

                        // Iterate over the CONST_REGEXES and check if the input string matches
                        for regex in CONST_REGEXES.iter() {
                            if let Some(captures) = regex.captures(&libfunc_id_str) {
                                if let Some(const_value) = captures.name("const") {
                                    // Convert string to a BigInt in order to decode it
                                    let const_value_str = const_value.as_str();
                                    let const_value_bigint =
                                        BigInt::parse_bytes(const_value_str.as_bytes(), 10)
                                            .unwrap();

                                    // If the const integer can be decoded to a valid string, use the string as a comment
                                    if let Some(decoded_string) =
                                        decode_hex_bigint(&const_value_bigint)
                                    {
                                        // Check if the string is not empty, not whitespace, and contains printable characters
                                        if !decoded_string.trim().is_empty()
                                            && decoded_string.chars().any(|c| c.is_ascii_graphic())
                                        {
                                            extracted_strings.insert(decoded_string);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Convert the extracted strings to a single string, separated by newline characters
        let result = extracted_strings
            .into_iter()
            .collect::<Vec<String>>()
            .join("\n");

        result
    }
}
