pub(crate) mod common;

mod backtrace;
mod build_profile;
mod build_trace_data;
mod clean;
mod collection;
mod color;
mod components;
mod contract_artifacts;
#[cfg(not(target_os = "windows"))]
// TODO(#2990): Run coverage test on Windows
mod coverage;
#[cfg(feature = "debugging")]
mod debugging;
mod docs_snippets_validation;
mod env;
mod features;
#[cfg(not(target_os = "windows"))]
mod fork_warning;
#[cfg(not(target_os = "windows"))]
mod forking;
mod fuzzing;
mod io_operations;
mod new;
mod requirements;
mod running;
mod steps;
mod trace_print;
mod trace_resources;
mod workspaces;
