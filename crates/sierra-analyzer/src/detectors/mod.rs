pub mod controlled_library_call_detector;
pub mod detector;
pub mod felt_overflow_detector;
pub mod functions_detector;
pub mod statistics_detector;
pub mod strings_detector;
pub mod tests_generator_detector;

use crate::detectors::controlled_library_call_detector::ControlledLibraryCallDetector;
use crate::detectors::detector::Detector;
use crate::detectors::felt_overflow_detector::FeltOverflowDetector;
use crate::detectors::functions_detector::FunctionsDetector;
use crate::detectors::statistics_detector::StatisticsDetector;
use crate::detectors::strings_detector::StringsDetector;
use crate::detectors::tests_generator_detector::TestsGeneratorDetector;

/// Macro to create a vector of detectors
macro_rules! create_detectors {
    ($($detector:ty),*) => {
        vec![
            $(
                Box::new(<$detector>::new()),
            )*
        ]
    };
}

/// Returns a vector of all the instantiated detectors
pub fn get_detectors() -> Vec<Box<dyn Detector>> {
    create_detectors!(
        FunctionsDetector,
        StringsDetector,
        StatisticsDetector,
        TestsGeneratorDetector,
        ControlledLibraryCallDetector,
        FeltOverflowDetector
    )
}
