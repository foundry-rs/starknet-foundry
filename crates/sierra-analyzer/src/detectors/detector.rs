use colored::Colorize;
use std::fmt::Debug;

use crate::decompiler::decompiler::Decompiler;

/// Possible types of a detector
#[derive(Debug, PartialEq)]
pub enum DetectorType {
    INFORMATIONAL,
    SECURITY,
    TESTING,
}

impl DetectorType {
    /// Returns the string representation of the DetectorType
    /// Used to print the detector type in the command-line tool
    pub fn as_str(&self) -> colored::ColoredString {
        match self {
            // Informational detectors types are green
            DetectorType::INFORMATIONAL => "Informational".normal().green(),

            // Security detectors types are blue
            DetectorType::SECURITY => "Security".normal().blue(),

            // Testing detectors types are yellow
            DetectorType::TESTING => "Testing".normal().yellow(),
        }
    }
}

/// Detector marker trait
pub trait Detector: Debug {
    // The id of a detector is used to select it using a command-line argument
    // e.g. the id of the detector with the name "Protoypes detector" is "prototypes"
    fn id(&self) -> &'static str;
    // Name of the detector
    fn name(&self) -> &'static str;
    // Description of the detector
    fn description(&self) -> &'static str;
    // A detector can be either a security detector or an informational detector
    fn detector_type(&self) -> DetectorType;
    // Run the detector on the decompiler instance
    fn detect(&mut self, decompiler: &mut Decompiler) -> String;
}
