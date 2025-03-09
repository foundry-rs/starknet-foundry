//! Crate with debugging utilities in forge.
//!
//! Currently, the only option that this crate gives is displaying pretty traces.
//! The entry point for that is the [`Trace`] struct that implements the [`Display`](std::fmt::Display)
//! which allows for pretty printing of traces.
mod trace;
mod tree;

pub use trace::Trace;
