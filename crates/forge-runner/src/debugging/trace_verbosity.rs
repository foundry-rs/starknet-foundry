use crate::debugging::component::Component;
use clap::ValueEnum;
use strum::VariantArray;

/// Trace verbosity level.
#[derive(ValueEnum, Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum TraceVerbosity {
    /// Display test name, contract name and selector.
    Minimal,
    /// Display test name, contract name, selector, calldata and call result.
    Standard,
    /// Display everything.
    Detailed,
}

impl TraceVerbosity {
    /// Converts the [`TraceVerbosity`] to a vector of [`Component`].
    #[must_use]
    pub fn to_components_vec(&self) -> Vec<&Component> {
        Component::VARIANTS
            .iter()
            .filter(|component| component.verbosity() <= *self)
            .collect()
    }
}
