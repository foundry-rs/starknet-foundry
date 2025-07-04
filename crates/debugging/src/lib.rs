//! Crate with debugging utilities in forge.
//!
//! Currently, the main purpose of this crate is displaying pretty traces.
//! The entry point for that is the [`Trace`] struct that implements the [`Display`](std::fmt::Display)
//! which allows for pretty printing of traces.
mod contracts_data_store;
mod trace;
mod tree;
mod verbosity;

pub use contracts_data_store::ContractsDataStore;
pub use trace::types::Trace;
pub use verbosity::Verbosity;
