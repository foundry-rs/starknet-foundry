use crate::decompiler::decompiler::Decompiler;
use crate::detectors::detector::{Detector, DetectorType};
use crate::sym_exec::sym_exec::generate_test_cases_for_function;

#[derive(Debug)]
pub struct TestsGeneratorDetector;

impl TestsGeneratorDetector {
    /// Creates a new `TestsGeneratorDetector` instance
    pub fn new() -> Self {
        Self
    }
}

impl Detector for TestsGeneratorDetector {
    /// Returns the id of the detector
    #[inline]
    fn id(&self) -> &'static str {
        "tests"
    }

    /// Returns the name of the detector
    #[inline]
    fn name(&self) -> &'static str {
        "Tests generator"
    }

    /// Returns the description of the detector
    #[inline]
    fn description(&self) -> &'static str {
        "Returns the tests cases for the functions."
    }

    /// Returns the type of the detector
    /// Detectors in the TESTING category are not displayed by default using the --detector flag
    #[inline]
    fn detector_type(&self) -> DetectorType {
        DetectorType::TESTING
    }

    /// Returns the generated unit tests for the function if they exist
    fn detect(&mut self, decompiler: &mut Decompiler) -> String {
        let mut result = String::new();

        for function in &mut decompiler.functions {
            // Determine the function name
            let function_name = if let Some(prototype) = &function.prototype {
                // Remove the "func " prefix and then split at the first parenthese
                let stripped_prototype = &prototype[5..];
                if let Some(first_space_index) = stripped_prototype.find('(') {
                    Some(stripped_prototype[..first_space_index].trim().to_string())
                } else {
                    None
                }
            } else {
                None
            };

            // If a function name was found, proceed with the mutable borrow
            if let Some(function_name) = function_name {
                // Add the test cases to the result
                let test_cases = generate_test_cases_for_function(
                    function,
                    decompiler.declared_libfuncs_names.clone(),
                );

                if !test_cases.is_empty() {
                    result += &format!("{} : \n", function_name);
                    result += &format!("{}\n", test_cases);
                }
            }
        }

        result
    }
}
