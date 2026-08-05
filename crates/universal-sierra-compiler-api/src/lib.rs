//! API for compiling Sierra programs using the `universal-sierra-compiler` (USC).
//!
//! This crate provides functions to compile Sierra JSON representations of contracts and raw programs into their respective Casm formats.
//!
//! # Note:
//! To allow more flexibility when changing internals, please make public as few items as possible.

pub use crate::cache::{
    CASM_CACHE_DIR, SierraProgramHash, contract_sierra_content_hash, raw_sierra_program_hash,
};
use crate::cache::{
    compile_sierra_at_path_with_content_hash, compile_sierra_bytes_with_content_hash,
};
use crate::command::{USCError, USCInternalCommand};
use crate::compile::{CompilationError, SierraType, compile_sierra, compile_sierra_at_path};
use crate::representation::RawCasmProgram;
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use serde_json::Value;
use std::path::Path;
use std::process::Command;

mod cache;
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

/// Compiles Sierra JSON bytes of a contract into [`CasmContractClass`],
/// reusing cached CASM under a precomputed Sierra content hash.
pub fn compile_contract_sierra_bytes_with_cache_key(
    sierra_bytes: &[u8],
    cache_dir: &Path,
    sierra_content_hash: &SierraProgramHash,
) -> Result<CasmContractClass, CompilationError> {
    compile_sierra_bytes_with_content_hash(
        sierra_bytes,
        SierraType::Contract,
        cache_dir,
        sierra_content_hash,
    )
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
/// reusing cached CASM under a precomputed Sierra content hash.
pub fn compile_raw_sierra_at_path_with_cache_key(
    sierra_file_path: &Path,
    cache_dir: &Path,
    sierra_content_hash: &SierraProgramHash,
) -> Result<RawCasmProgram, CompilationError> {
    compile_sierra_at_path_with_content_hash(
        sierra_file_path,
        SierraType::Raw,
        cache_dir,
        sierra_content_hash,
    )
}

/// Creates a `universal-sierra-compiler --version` command.
///
/// Only exists because of how requirements checker was implemented.
pub fn version_command() -> Result<Command, USCError> {
    Ok(USCInternalCommand::new()?.arg("--version").command())
}
