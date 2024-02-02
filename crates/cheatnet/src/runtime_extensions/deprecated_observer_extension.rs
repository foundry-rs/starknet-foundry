use super::{
    deprecated_cheatable_starknet_extension::runtime::{
        DeprecatedExtendedRuntime, DeprecatedExtensionLogic, DeprecatedStarknetRuntime,
    },
    observer_extension::ObserverState,
};

pub struct DeprecatedObserverExtension<'a> {
    pub observer_state: &'a mut ObserverState,
}

impl<'a> DeprecatedObserverExtension<'a> {
    pub fn from(observer_state: &'a mut ObserverState) -> Self {
        DeprecatedObserverExtension { observer_state }
    }
}

pub type DeprecatedObserverRuntime<'a> = DeprecatedExtendedRuntime<DeprecatedObserverExtension<'a>>;

impl<'a> DeprecatedExtensionLogic for DeprecatedObserverExtension<'a> {
    type Runtime = DeprecatedStarknetRuntime<'a>;
    // Currently it exists only to pass state to nested runtimes
    // Cairo 0 runtimes are not observed
}
