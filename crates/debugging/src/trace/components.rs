use crate::trace::types::{
    CallerAddress, ContractAddress, ContractName, TransformedCallResult, TransformedCalldata,
};
use blockifier::execution::entry_point::CallType;
use paste::paste;
use starknet_api::contract_class::EntryPointType;
use std::collections::HashSet;

/// Represents a set of [`Component`] that will be included in a trace.
pub struct Components {
    set: HashSet<Component>,
}

impl Components {
    /// Creates a new [`Components`] instance with the specified set of [`Component`].
    #[must_use]
    pub fn new(components: HashSet<Component>) -> Self {
        Self { set: components }
    }

    /// Checks if a specific [`Component`] is included in the set.
    #[must_use]
    pub fn contains(&self, component: &Component) -> bool {
        self.set.contains(component)
    }
}

/// Components that will be included in the trace.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
}

macro_rules! impl_component_container {
    // Short form - uses the same name for both the Component variant and the value type
    ($variant:ident) => {
        impl_component_container!($variant, $variant);
    };

    // Full form - Component name and different value type
    ($variant:ident, $ty:ty) => {
        paste! {
            #[doc = concat!(
                "Container for a `", stringify!($ty),
                "` that is computed only if `Component::", stringify!($variant),
                "` is included in the [`Components`]."
            )]
            #[derive(Debug, Clone)]
            pub struct [<$variant Container>] {
                value: Option<$ty>,
            }

            impl [<$variant Container>] {
                #[doc = concat!(
                    "Creates a new [`", stringify!($variant), "Container`] from a specific value.\n\n",
                    "This will store the value only if `Component::", stringify!($variant), "` is present in the [`Components`]."
                )]
                #[must_use]
                pub fn new(components: &Components, value: $ty) -> Self {
                    Self {
                        value: components.contains(&Component::$variant).then_some(value),
                    }
                }

                #[doc = concat!(
                    "Creates a new [`", stringify!($variant), "Container`] using a lazy supplier function.\n\n",
                    "The function will be called only if `Component::", stringify!($variant), "` is present in [`Components`]."
                )]
                #[must_use]
                pub fn new_lazy(components: &Components, supply_fn: impl FnOnce() -> $ty) -> Self {
                    Self {
                        value: components.contains(&Component::$variant).then(supply_fn),
                    }
                }

                #[doc = "Returns a reference to the contained value if it was computed.\n\nReturns `None` otherwise."]
                #[must_use]
                pub fn as_option(&self) -> Option<&$ty> {
                    self.value.as_ref()
                }
            }

            impl Components {
                #[doc = concat!(
                    "Returns a [`", stringify!($variant), "Container`] from a direct value.\n\n",
                    "The value will only be stored if `Component::", stringify!($variant), "` is in the set."
                )]
                #[must_use]
                pub fn [<$variant:snake>](&self, value: $ty) -> [<$variant Container>] {
                    [<$variant Container>]::new(self, value)
                }

                #[doc = concat!(
                    "Returns a [`", stringify!($variant), "Container`] using a lazy supplier function.\n\n",
                    "The function will only be called if `Component::", stringify!($variant), "` is in the set."
                )]
                pub fn [<$variant:snake _lazy>](&self, supply_fn: impl FnOnce() -> $ty) -> [<$variant Container>] {
                    [<$variant Container>]::new_lazy(self, supply_fn)
                }
            }
        }
    };
}

impl_component_container!(ContractName);
impl_component_container!(EntryPointType);
impl_component_container!(Calldata, TransformedCalldata);
impl_component_container!(ContractAddress);
impl_component_container!(CallerAddress);
impl_component_container!(CallType);
impl_component_container!(CallResult, TransformedCallResult);
