use crate::decompiler::decompiler::Decompiler;
use crate::detectors::detector::{Detector, DetectorType};

#[derive(Debug)]
pub struct StatisticsDetector;

impl StatisticsDetector {
    /// Creates a new `StatisticsDetector` instance
    pub fn new() -> Self {
        Self
    }
}

impl Detector for StatisticsDetector {
    /// Returns the id of the detector
    #[inline]
    fn id(&self) -> &'static str {
        "statistics"
    }

    /// Returns the name of the detector
    #[inline]
    fn name(&self) -> &'static str {
        "Program Statistics"
    }

    /// Returns the description of the detector
    #[inline]
    fn description(&self) -> &'static str {
        "Returns the functions statistics."
    }

    /// Returns the type of the detector
    #[inline]
    fn detector_type(&self) -> DetectorType {
        DetectorType::INFORMATIONAL
    }

    /// Returns all the functions statistics
    fn detect(&mut self, decompiler: &mut Decompiler) -> String {
        let sierra_program = decompiler.sierra_program.program();

        let libfuncs_len = sierra_program.libfunc_declarations.len();
        let types_len = sierra_program.type_declarations.len();
        let functions_len = sierra_program.funcs.len();

        let result = format!(
            "Libfuncs: {}\nTypes: {}\nFunctions: {}",
            libfuncs_len, types_len, functions_len
        );

        result
    }
}
