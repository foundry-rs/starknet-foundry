use cairo_lang_sierra::extensions::core::CoreConcreteLibfunc;
use cairo_lang_sierra::extensions::lib_func::ParamSignature;
use cairo_lang_sierra::extensions::starknet::StarkNetConcreteLibfunc;
use cairo_lang_sierra::ids::VarId;
use cairo_lang_sierra::program::GenStatement;

use crate::decompiler::decompiler::Decompiler;
use crate::detectors::detector::{Detector, DetectorType};
use crate::parse_element_name;

#[derive(Debug)]
pub struct ControlledLibraryCallDetector;

impl ControlledLibraryCallDetector {
    /// Creates a new `ControlledLibraryCallDetector` instance
    pub fn new() -> Self {
        Self
    }
}

pub const BUILTINS: [&str; 8] = [
    "Pedersen",
    "RangeCheck",
    "Bitwise",
    "EcOp",
    "Poseidon",
    "SegmentArena",
    "GasBuiltin",
    "System",
];

/// Filter the builtins arguments and returns only the user defined arguments
fn filter_builtins_from_arguments(
    signature: &[ParamSignature],
    arguments: Vec<VarId>,
) -> Vec<VarId> {
    signature
        .iter()
        .zip(arguments)
        .filter(|(signature_elem, _)| {
            !BUILTINS.contains(&signature_elem.ty.debug_name.as_ref().unwrap().as_str())
        })
        .map(|(_, argument_element)| argument_element)
        .collect()
}

fn check_user_controlled(
    formal_params: &[ParamSignature],
    actual_params: Vec<VarId>,
    _function_name: &str,
) -> bool {
    // The first argument is the class hash
    let _class_hash = filter_builtins_from_arguments(formal_params, actual_params)[0].clone();

    // TODO : Check if the class hash argument is tainted or not
    true
}

impl Detector for ControlledLibraryCallDetector {
    /// Returns the id of the detector
    #[inline]
    fn id(&self) -> &'static str {
        "controlled_library_call"
    }

    /// Returns the name of the detector
    #[inline]
    fn name(&self) -> &'static str {
        "Controlled library call"
    }

    /// Returns the description of the detector
    #[inline]
    fn description(&self) -> &'static str {
        "Detect library calls with a user controlled class hash."
    }

    /// Returns the type of the detector
    #[inline]
    fn detector_type(&self) -> DetectorType {
        DetectorType::SECURITY
    }

    /// Detect library calls with a user controlled class hash
    fn detect(&mut self, decompiler: &mut Decompiler) -> String {
        let mut result = String::new();

        for function in decompiler.user_defined_functions() {
            for statement in function.library_functions_calls.clone() {
                if let GenStatement::Invocation(statement) = statement.statement {
                    let libfunc = decompiler
                        .registry()
                        .get_libfunc(&statement.libfunc_id)
                        .expect("Library function not found in the registry");

                    if let CoreConcreteLibfunc::FunctionCall(abi_function) = libfunc {
                        if check_user_controlled(
                            &abi_function.signature.param_signatures,
                            statement.args.clone(),
                            parse_element_name!(function.function.id.clone()).as_str(),
                        ) {
                            result += &format!(
                                "{} in {}",
                                statement,
                                parse_element_name!(function.function.id.clone()).as_str()
                            )
                            .to_string();
                        };
                    }
                }
            }

            for statement in function.statements.clone() {
                if let GenStatement::Invocation(statement) = statement.statement {
                    let libfunc = decompiler
                        .registry()
                        .get_libfunc(&statement.libfunc_id)
                        .expect("Library function not found in the registry");

                    // We care only about a library call
                    if let CoreConcreteLibfunc::StarkNet(StarkNetConcreteLibfunc::LibraryCall(l)) =
                        libfunc
                    {
                        if check_user_controlled(
                            &l.signature.param_signatures,
                            statement.args.clone(),
                            parse_element_name!(function.function.id.clone()).as_str(),
                        ) {
                            result += &format!(
                                "{} in {}",
                                statement,
                                parse_element_name!(function.function.id.clone()).as_str()
                            )
                            .to_string();
                        };
                    }
                }
            }
        }

        result
    }
}
