use crate::command::{USCError, USCInternalCommand};
use serde_json::Value;
use std::io;
use std::io::Write;
use std::path::Path;
use strum_macros::Display;
use tempfile::Builder;
use thiserror::Error;

/// Errors that can occur during Sierra compilation.
#[derive(Debug, Error)]
pub enum CompilationError {
    #[error("Failed to write Sierra JSON to temp file: {0}")]
    TempFileWrite(#[from] io::Error),

    #[error("Could not serialize Sierra JSON: {0}")]
    Serialization(serde_json::Error),

    #[error(transparent)]
    USCSetup(#[from] USCError),

    #[error("Failed to deserialize compilation output: {0}")]
    Deserialization(serde_json::Error),
}

#[derive(Debug, Display, Copy, Clone)]
#[strum(serialize_all = "lowercase")]
pub enum SierraType {
    Contract,
    Raw,
}

/// Compiles the given Sierra JSON into the specified type using the `universal-sierra-compiler`.
#[tracing::instrument(skip_all, level = "debug")]
pub fn compile_sierra(
    sierra_json: &Value,
    sierra_type: SierraType,
) -> Result<String, CompilationError> {
    let mut temp_sierra_file = Builder::new().tempfile()?;

    let json_bytes = serde_json::to_vec(sierra_json).map_err(CompilationError::Serialization)?;
    temp_sierra_file.write_all(&json_bytes)?;

    compile_sierra_at_path(temp_sierra_file.path(), sierra_type)
}

/// Compiles the Sierra file at the given path into the specified type using the `universal-sierra-compiler`.
#[tracing::instrument(skip_all, level = "debug")]
pub fn compile_sierra_at_path(
    sierra_file_path: &Path,
    sierra_type: SierraType,
) -> Result<String, CompilationError> {
    let usc_output = USCInternalCommand::new()?
        .arg(format!("compile-{sierra_type}"))
        .arg("--sierra-path")
        .arg(sierra_file_path)
        .run()?;

    Ok(String::from_utf8(usc_output.stdout).expect("valid UTF-8 from universal-sierra-compiler"))
}
