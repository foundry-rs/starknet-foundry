use crate::debugging::trace_verbosity::TraceVerbosity;
use clap::ValueEnum;
use strum_macros::VariantArray;

/// Components that will be included in the trace.
#[derive(ValueEnum, Clone, VariantArray, Debug, Eq, PartialEq)]
pub enum Component {
    /// The name of the contract being called.
    ContractName,
    /// The type of the entry point being called (e.g., `External`, `L1Handler`, etc.).
    EntryPointType,
    /// The calldata of the call, transformed for display.
    Calldata,
    /// The address of the contract being called.
    ContractAddress,
    /// The address of the caller contract.
    CallerAddress,
    /// The type of the call (e.g., `Call`, `Delegate`, etc.).
    CallType,
    /// The result of the call, transformed for display.
    CallResult,
    /// The function trace, will be only shown if the contract is not a fork.
    FunctionTrace,
}
impl Component {
    /// Returns minimal [`TraceVerbosity`] for the component.
    #[must_use]
    pub fn verbosity(&self) -> TraceVerbosity {
        match self {
            Component::ContractName => TraceVerbosity::Minimal,
            Component::Calldata | Component::CallResult => TraceVerbosity::Standard,
            Component::ContractAddress
            | Component::CallerAddress
            | Component::EntryPointType
            | Component::CallType
            | Component::FunctionTrace => TraceVerbosity::Detailed,
        }
    }
}

impl From<&Component> for debugging::Component {
    fn from(component: &Component) -> Self {
        match component {
            Component::ContractName => debugging::Component::ContractName,
            Component::EntryPointType => debugging::Component::EntryPointType,
            Component::Calldata => debugging::Component::Calldata,
            Component::ContractAddress => debugging::Component::ContractAddress,
            Component::CallerAddress => debugging::Component::CallerAddress,
            Component::CallType => debugging::Component::CallType,
            Component::CallResult => debugging::Component::CallResult,
            Component::FunctionTrace => debugging::Component::FunctionTrace,
        }
    }
}
