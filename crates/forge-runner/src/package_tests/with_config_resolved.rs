use super::{TestCase, TestTarget};
use crate::expected_result::ExpectedTestResult;
use crate::running::hints_to_params;
use anyhow::Result;
use cairo_vm::serde::deserialize_program::ReferenceManager;
use cairo_vm::types::program::Program;
use cairo_vm::types::relocatable::MaybeRelocatable;
use cheatnet::runtime_extensions::forge_config_extension::config::{
    RawAvailableGasConfig, RawFuzzerConfig,
};
use starknet_api::block::BlockNumber;
use starknet_types_core::felt::Felt;
use std::collections::HashMap;
use universal_sierra_compiler_api::AssembledProgramWithDebugInfo;
use url::Url;

pub type TestTargetWithResolvedConfig = TestTarget<TestCaseResolvedConfig>;

pub type TestCaseWithResolvedConfig = TestCase<TestCaseResolvedConfig>;

impl TestCaseWithResolvedConfig {
    pub fn try_into_program(
        &self,
        casm_program: &AssembledProgramWithDebugInfo,
    ) -> Result<Program> {
        let builtins = self.test_details.builtins();

        let assembled_program = &casm_program.assembled_cairo_program;
        let hints_dict = hints_to_params(assembled_program);
        let data: Vec<MaybeRelocatable> = assembled_program
            .bytecode
            .iter()
            .map(Felt::from)
            .map(MaybeRelocatable::from)
            .collect();

        Program::new(
            builtins.clone(),
            data,
            Some(0),
            hints_dict,
            ReferenceManager {
                references: Vec::new(),
            },
            HashMap::new(),
            vec![],
            None,
        )
        .map_err(std::convert::Into::into)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedForkConfig {
    pub url: Url,
    pub block_number: BlockNumber,
}

/// Test case with config that has been resolved, that is
///     `#[fork("name")]` -> url and block id
///     fetches block number
#[derive(Debug, Clone, PartialEq)]
pub struct TestCaseResolvedConfig {
    pub available_gas: Option<RawAvailableGasConfig>,
    pub ignored: bool,
    pub expected_result: ExpectedTestResult,
    pub fork_config: Option<ResolvedForkConfig>,
    pub fuzzer_config: Option<RawFuzzerConfig>,
    pub disable_predeployed_contracts: bool,
}
