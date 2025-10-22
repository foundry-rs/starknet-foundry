use crate::Components;
use crate::contracts_data_store::ContractsDataStore;

/// Context is a structure that holds the necessary data for creating a [`Trace`](crate::Trace).
pub struct Context {
    contracts_data_store: ContractsDataStore,
    components: Components,
}

impl Context {
    /// Creates a new instance of [`Context`] from a given `cheatnet` [`ContractsData`], [`ForkData`] and [`Components`].
    #[must_use]
    pub fn new(contracts_data_store: ContractsDataStore, components: Components) -> Self {
        Self {
            contracts_data_store,
            components,
        }
    }

    /// Returns a reference to the [`ContractsDataStore`].
    #[must_use]
    pub fn contracts_data_store(&self) -> &ContractsDataStore {
        &self.contracts_data_store
    }

    /// Returns a reference to the [`Components`].
    #[must_use]
    pub fn components(&self) -> &Components {
        &self.components
    }
}
