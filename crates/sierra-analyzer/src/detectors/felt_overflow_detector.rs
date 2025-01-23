use crate::decompiler::decompiler::Decompiler;
use crate::decompiler::function::FunctionType;
use crate::decompiler::libfuncs_patterns::{
    ADDITION_REGEX, MULTIPLICATION_REGEX, SUBSTRACTION_REGEX,
};
use crate::detectors::detector::{Detector, DetectorType};
use crate::parse_element_name_with_fallback;
use crate::var_id_to_name;

use cairo_lang_sierra::program::GenStatement;

#[derive(Debug)]
pub struct FeltOverflowDetector;

impl FeltOverflowDetector {
    /// Creates a new `FeltOverflowDetector` instance
    pub fn new() -> Self {
        Self
    }
}

impl Detector for FeltOverflowDetector {
    /// Returns the id of the detector
    #[inline]
    fn id(&self) -> &'static str {
        "felt_overflow"
    }

    /// Returns the name of the detector
    #[inline]
    fn name(&self) -> &'static str {
        "Felt Overflow"
    }

    /// Returns the description of the detector
    #[inline]
    fn description(&self) -> &'static str {
        "Detects the potential felt overflows."
    }

    /// Returns the type of the detector
    #[inline]
    fn detector_type(&self) -> DetectorType {
        DetectorType::SECURITY
    }

    /// Returns all the functions names
    fn detect(&mut self, decompiler: &mut Decompiler) -> String {
        let mut result = String::new();
        let mut found_vulnerabilities = Vec::new();

        // We extract the functions names from the prototypes
        decompiler.decompile_functions_prototypes();

        for function in decompiler.functions.clone() {
            // Skip core functions
            if let Some(function_type) = function.function_type {
                if matches!(function_type, FunctionType::Core) {
                    continue;
                }
            }

            let function_name = function.function.id.clone();

            let arguments = function.arguments;

            // Filter arguments felt arguments
            let felt_arguments: Vec<_> = arguments
                .iter()
                .filter(|&&(_, ref arg_type)| arg_type == "felt252")
                .collect();

            for statement in function.statements {
                if let GenStatement::Invocation(invocation) = &statement.statement {
                    let arguments = invocation.args.clone();
                    let mut local_found_felt_arguments = Vec::new();

                    for argument in arguments {
                        let element_name = var_id_to_name!(argument);

                        // Check if the argument is in the felt_arguments
                        if felt_arguments
                            .iter()
                            .any(|&(ref arg_name, _)| arg_name == &element_name)
                        {
                            local_found_felt_arguments.push(element_name);
                        }
                    }

                    // Parse the libfunc name used in the statement
                    let libfunc_name = parse_element_name_with_fallback!(
                        invocation.libfunc_id,
                        decompiler.declared_libfuncs_names
                    );

                    // Detect if we perform an arithmetic operation with a felt argument
                    if ADDITION_REGEX.is_match(&libfunc_name)
                        || SUBSTRACTION_REGEX.is_match(&libfunc_name)
                        || MULTIPLICATION_REGEX.is_match(&libfunc_name)
                    {
                        if !local_found_felt_arguments.is_empty() {
                            found_vulnerabilities
                                .push((function_name.clone(), local_found_felt_arguments));
                        }
                    }
                }
            }
        }

        // Append the found vulnerabilities to the result
        if !found_vulnerabilities.is_empty() {
            for (function_name, arguments) in found_vulnerabilities {
                let arguments_str = arguments.join(", ");
                result.push_str(&format!(
                    "{}: parameters {} could be used to trigger a felt overflow/underflow\n",
                    function_name, arguments_str
                ));
            }
        }

        result
    }
}
