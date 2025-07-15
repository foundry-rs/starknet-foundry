pub(crate) mod common;

mod backtrace;
mod build_profile;
mod build_trace_data;
mod clean;
mod collection;
mod color;
mod completions;
mod components;
mod contract_artifacts;
mod coverage;
#[cfg(feature = "debugging")]
mod debugging;
mod docs_snippets_validation;
mod env;
mod features;
mod fork_warning;
mod forking;
mod fuzzing;
mod io_operations;
mod new;
mod requirements;
mod running;
mod steps;
mod templates;
mod trace_print;
mod trace_resources;
mod workspaces;
