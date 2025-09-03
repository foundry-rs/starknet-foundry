//! This module provides functionality related to `cairo-native`.
//!
//! Currently, it includes:
//!  - Compiling Sierra contract classes into `cairo-native` executors.

use cairo_lang_starknet_classes::contract_class::ContractClass;
use cairo_native::executor::AotContractExecutor;
use starknet_api::contract_class::SierraVersion;

/// Compiles a given Sierra [`ContractClass`] into an [`AotContractExecutor`] for `cairo-native` execution.
#[must_use]
pub fn compile_contract_class(contract_class: &ContractClass) -> AotContractExecutor {
    let sierra_program = contract_class
        .extract_sierra_program()
        .expect("extraction should succeed");

    let sierra_version = extract_sierra_version(contract_class);

    AotContractExecutor::new(
        &sierra_program,
        &contract_class.entry_points_by_type,
        sierra_version.clone().into(),
        cairo_native::OptLevel::Default,
        None,
    )
    .expect("compilation should succeed")
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
