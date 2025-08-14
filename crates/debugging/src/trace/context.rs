use crate::Components;
use crate::contracts_data_store::ContractsDataStore;
use cheatnet::forking::data::ForkData;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;

/// Context is a structure that holds the necessary data for creating a [`Trace`](crate::Trace).
pub struct Context {
    contracts_data_store: ContractsDataStore,
    components: Components,
}

impl Context {
    /// Creates a new instance of [`Context`] from a given `cheatnet` [`ContractsData`], [`ForkData`] and [`Components`].
    #[must_use]
    pub fn new(
        contracts_data: &ContractsData,
        fork_data: &ForkData,
        components: Components,
    ) -> Self {
        let contracts_data_store = ContractsDataStore::new(contracts_data, fork_data);
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
