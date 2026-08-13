//! API for compiling Sierra programs using the `universal-sierra-compiler` (USC).
//!
//! This crate provides functions to compile Sierra JSON representations of contracts and raw programs into their respective Casm formats.
//!
//! # Note:
//! To allow more flexibility when changing internals, please make public as few items as possible.

use crate::command::{USCError, USCInternalCommand};
use crate::compile::{
    CompilationError, SierraType, compile_sierra, compile_sierra_at_path,
    compile_sierra_at_path_with_cache_dir,
};
use crate::representation::RawCasmProgram;
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use serde_json::Value;
use std::path::Path;
use std::process::Command;

mod command;
mod compile;
pub mod representation;

pub use crate::command::supports_cache_dir;

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

/// Compiles Sierra JSON file at the given path of a contract into [`CasmContractClass`],
/// reusing the compiler-managed cache when `cache_dir` is provided.
pub fn compile_contract_sierra_at_path_with_cache_dir(
    sierra_file_path: &Path,
    cache_dir: Option<&Path>,
) -> Result<CasmContractClass, CompilationError> {
    let json =
        compile_sierra_at_path_with_cache_dir(sierra_file_path, SierraType::Contract, cache_dir)?;
    serde_json::from_str(&json).map_err(CompilationError::Deserialization)
}

/// Compiles Sierra JSON of a raw program into [`RawCasmProgram`].
pub fn compile_raw_sierra(sierra_json: &Value) -> Result<RawCasmProgram, CompilationError> {
    let json = compile_sierra(sierra_json, SierraType::Raw)?;
    serde_json::from_str(&json).map_err(CompilationError::Deserialization)
}

/// Compiles Sierra JSON file at the given path of a raw program into [`RawCasmProgram`].
pub fn compile_raw_sierra_at_path(
    sierra_file_path: &Path,
) -> Result<RawCasmProgram, CompilationError> {
    let json = compile_sierra_at_path(sierra_file_path, SierraType::Raw)?;
    serde_json::from_str(&json).map_err(CompilationError::Deserialization)
}

/// Compiles Sierra JSON file at the given path of a raw program into [`RawCasmProgram`],
/// reusing the compiler-managed cache when `cache_dir` is provided.
pub fn compile_raw_sierra_at_path_with_cache_dir(
    sierra_file_path: &Path,
    cache_dir: Option<&Path>,
) -> Result<RawCasmProgram, CompilationError> {
    let json = compile_sierra_at_path_with_cache_dir(sierra_file_path, SierraType::Raw, cache_dir)?;
    serde_json::from_str(&json).map_err(CompilationError::Deserialization)
}

/// Creates a `universal-sierra-compiler --version` command.
///
/// Only exists because of how requirements checker was implemented.
pub fn version_command() -> Result<Command, USCError> {
    Ok(USCInternalCommand::new()?.arg("--version").command())
}
