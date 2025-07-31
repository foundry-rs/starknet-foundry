use crate::debugging::component::Component;
use clap::ValueEnum;
use debugging::Components;
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
    /// Converts the [`TraceVerbosity`] to a [`Components`] that will be included in the trace.
    #[must_use]
    pub fn to_components(&self) -> Components {
        Components::new(
            Component::VARIANTS
                .iter()
                .filter(|component| component.verbosity() <= *self)
                .cloned()
                .map(debugging::Component::from)
                .collect(),
        )
    }
}
