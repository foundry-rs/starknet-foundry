//! This module provides functionality related to `cairo-native`.
//!
//! Currently, it includes:
//!  - Compiling Sierra contract classes into `cairo-native` executors.

use cairo_lang_starknet_classes::contract_class::ContractClass;
use cairo_native::executor::AotContractExecutor;
use starknet_api::contract_class::SierraVersion;

#[derive(Debug, thiserror::Error)]
pub enum NativeCompilationError {
    #[error("Unsupported Sierra version {0}. cairo-native requires version 1.7.0 or later.")]
    UnsupportedSierraVersion(String),
}

/// Compiles a given Sierra [`ContractClass`] into an [`AotContractExecutor`] for `cairo-native` execution.
pub fn compile_contract_class(
    contract_class: &ContractClass,
) -> Result<AotContractExecutor, NativeCompilationError> {
    let sierra_version = extract_sierra_version(contract_class);
    check_sierra_version(&sierra_version)?;

    let sierra_program = contract_class
        .extract_sierra_program()
        .expect("extraction should succeed");

    Ok(AotContractExecutor::new(
        &sierra_program,
        &contract_class.entry_points_by_type,
        sierra_version.clone().into(),
        cairo_native::OptLevel::Default,
        None,
    )
    .expect("compilation should succeed"))
}

/// Extracts the Sierra version from the given [`ContractClass`].
fn extract_sierra_version(contract_class: &ContractClass) -> SierraVersion {
    let sierra_version_values = contract_class
        .sierra_program
        .iter()
        .take(3)
        .map(|x| x.value.clone())
        .collect::<Vec<_>>();

    SierraVersion::extract_from_program(&sierra_version_values)
        .expect("version extraction should succeed")
}

/// Checks if the given Sierra version is supported by `cairo-native`.
fn check_sierra_version(sierra_version: &SierraVersion) -> Result<(), NativeCompilationError> {
    let minimal_supported_version = SierraVersion::new(1, 7, 0);
    if sierra_version < &minimal_supported_version {
        Err(NativeCompilationError::UnsupportedSierraVersion(
            sierra_version.to_string(),
        ))
    } else {
        Ok(())
    }
}
