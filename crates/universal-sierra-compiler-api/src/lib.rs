//! API for compiling Sierra programs using the `universal-sierra-compiler` (USC).
//!
//! This crate provides functions to compile Sierra JSON representations of contracts and raw programs into their respective Casm formats.
//!
//! # Note:
//! To allow more flexibility when changing internals, please make public as few items as possible.

use crate::command::{USCError, USCInternalCommand};
use crate::compile::{CompilationError, SierraType, compile_sierra, compile_sierra_at_path};
use crate::representation::RawCasmProgram;
use anyhow::bail;
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use tracing::trace_span;
use uv_once_map::OnceMap;

mod command;
mod compile;
pub mod representation;

/// Compiles Sierra JSON of a contract into [`CasmContractClass`].
pub fn compile_contract_sierra(sierra_json: &Value) -> Result<CasmContractClass, CompilationError> {
    let json = compile_sierra(sierra_json, SierraType::Contract)?;
    serde_json::from_str(&json).map_err(CompilationError::Deserialization)
}

/// Compiles Sierra JSON file at the given path of a contract into [`CasmContractClass`].
pub fn compile_contract_sierra_at_path(
    sierra_file_path: &Path,
) -> Result<CasmContractClass, CompilationError> {
    let json = compile_sierra_at_path(sierra_file_path, SierraType::Contract)?;
    serde_json::from_str(&json).map_err(CompilationError::Deserialization)
}

/// Compiles Sierra JSON of a raw program into [`RawCasmProgram`].
pub fn compile_raw_sierra(sierra_json: &Value) -> Result<RawCasmProgram, CompilationError> {
    let json = compile_sierra(sierra_json, SierraType::Raw)?;
    serde_json::from_str(&json).map_err(CompilationError::Deserialization)
}

static SIERRA_RAW_COMPILATION_DATA_LOADER: OnceLock<
    OnceMap<PathBuf, Result<RawCasmProgram, CompilationErrorString>>,
> = OnceLock::new();

// We need a cloneable error type for the `OnceMap` to work.
#[derive(Clone)]
struct CompilationErrorString(String);

/// Starts a compilation of Sierra JSON file at the given path into [`RawCasmProgram`].
/// This is a fire and forget operation. It neither blocks the thread, nor waits for compilation
/// to finish in any way.
/// The compilation will happen on another thread.
pub fn schedule_compile_raw_sierra_at_path(sierra_file_path: &Path) -> anyhow::Result<()> {
    let cell = SIERRA_RAW_COMPILATION_DATA_LOADER.get_or_init(OnceMap::default);
    let path = sierra_file_path.to_path_buf();

    if cell.register(path.clone()) {
        rayon::spawn(move || {
            let span = trace_span!("compile_raw_sierra_at_path");
            let result = {
                let _g = span.enter();
                compile_sierra_at_path(&path, SierraType::Raw).and_then(|json| {
                    serde_json::from_str(&json).map_err(CompilationError::Deserialization)
                })
            };
            match result {
                Ok(program) => {
                    cell.done(path, Ok(program));
                }
                Err(e) => {
                    cell.done(path, Err(CompilationErrorString(e.to_string())));
                }
            }
        });
    }

    Ok(())
}

/// Waits in a blocking manner for a compilation of Sierra JSON file at the given path to finish.
/// Returns the raw program compiled into [`RawCasmProgram`].
/// Panics unless `schedule_compile_raw_sierra_at_path` has been called before.
pub fn blocking_get_compiled_raw_sierra_at_path(
    sierra_file_path: &Path,
) -> anyhow::Result<RawCasmProgram> {
    let cell = SIERRA_RAW_COMPILATION_DATA_LOADER.get_or_init(OnceMap::default);
    let path = sierra_file_path.to_path_buf();
    let span = trace_span!("waiting_for_compiled_raw_sierra_at_path");
    let result = {
        let _g = span.enter();
        cell.wait_blocking(&path)
            .expect("schedule_compile_raw_sierra_at_path must be called first")
    };
    match result {
        Ok(metadata) => Ok(metadata),
        Err(e) => bail!(e.0),
    }
}

/// Creates a `universal-sierra-compiler --version` command.
///
/// Only exists because of how requirements checker was implemented.
pub fn version_command() -> Result<Command, USCError> {
    Ok(USCInternalCommand::new()?.arg("--version").command())
}
