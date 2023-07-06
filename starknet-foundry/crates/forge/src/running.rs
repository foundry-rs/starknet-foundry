use std::collections::HashMap;

use anyhow::Result;
use cairo_vm::serde::deserialize_program::HintParams;
use cheatable_starknet::constants::build_testing_state;
use itertools::chain;

use cairo_lang_casm::hints::Hint;
use cairo_lang_casm::instructions::Instruction;
use cairo_lang_runner::casm_run::hint_to_hint_params;
use cairo_lang_runner::{
    CairoHintProcessor as CoreCairoHintProcessor, RunResultValue, RunnerError,
};
use cairo_lang_runner::{RunResult, SierraCasmRunner, StarknetState};
use cairo_vm::vm::runners::cairo_runner::RunResources;
use test_collector::TestUnit;

use crate::cheatcodes_hint_processor::CairoHintProcessor;
use crate::scarb::StarknetContractArtifacts;
use crate::test_results::{extract_result_data, TestResult};

/// Builds `hints_dict` required in `cairo_vm::types::program::Program` from instructions.
fn build_hints_dict<'b>(
    instructions: impl Iterator<Item = &'b Instruction>,
) -> (HashMap<usize, Vec<HintParams>>, HashMap<String, Hint>) {
    let mut hints_dict: HashMap<usize, Vec<HintParams>> = HashMap::new();
    let mut string_to_hint: HashMap<String, Hint> = HashMap::new();

    let mut hint_offset = 0;

    for instruction in instructions {
        if !instruction.hints.is_empty() {
            // Register hint with string for the hint processor.
            for hint in &instruction.hints {
                string_to_hint.insert(format!("{hint:?}"), hint.clone());
            }
            // Add hint, associated with the instruction offset.
            hints_dict.insert(
                hint_offset,
                instruction.hints.iter().map(hint_to_hint_params).collect(),
            );
        }
        hint_offset += instruction.body.op_size();
    }
    (hints_dict, string_to_hint)
}

type TestUnitExitStatus = RunResultValue;

#[derive(Debug, PartialEq, Clone)]
pub struct TestUnitSummary {
    pub test_unit: TestUnit,
    pub gas_counter: Option<cairo_felt::Felt252>,
    pub memory: Vec<Option<cairo_felt::Felt252>>,
    pub exit_status: TestUnitExitStatus,
}

impl TestUnitSummary {
    fn from(test_unit: TestUnit, run_result: RunResult) -> Self {
        Self {
            test_unit,
            gas_counter: run_result.gas_counter,
            memory: run_result.memory,
            exit_status: run_result.value,
        }
    }

    #[must_use]
    pub fn passed(&self) -> bool {
        match self.exit_status {
            TestUnitExitStatus::Success(_) => true,
            TestUnitExitStatus::Panic(_) => false,
        }
    }
}

fn test_result_from_run_result(name: &str, run_result: RunResult) -> TestResult {
    match run_result.value {
        RunResultValue::Success(_) => TestResult::Passed {
            name: name.to_string(),
            msg: extract_result_data(&run_result),
            run_result: Some(run_result),
        },
        RunResultValue::Panic(_) => TestResult::Failed {
            name: name.to_string(),
            msg: extract_result_data(&run_result),
            run_result: Some(run_result),
        },
    }
}

pub(crate) fn run_from_test_units(
    runner: &mut SierraCasmRunner,
    unit: &TestUnit,
    contracts: &HashMap<String, StarknetContractArtifacts>,
) -> Result<TestResult> {
    let available_gas = if let Some(available_gas) = &unit.available_gas {
        Some(*available_gas)
    } else {
        Some(usize::MAX)
    };
    let func = runner.find_function(unit.name.as_str())?;
    let initial_gas = runner.get_initial_available_gas(func, available_gas)?;
    let (entry_code, builtins) = runner.create_entry_code(func, &[], initial_gas)?;
    let footer = runner.create_code_footer();
    let instructions = chain!(
        entry_code.iter(),
        runner.get_casm_program().instructions.iter(),
        footer.iter()
    );
    let (hints_dict, string_to_hint) = build_hints_dict(instructions.clone());
    let core_cairo_hint_processor = CoreCairoHintProcessor {
        runner: Some(runner),
        starknet_state: StarknetState::default(),
        string_to_hint,
        blockifier_state: None,
        run_resources: RunResources::default(),
    };
    let mut cairo_hint_processor = CairoHintProcessor {
        original_cairo_hint_processor: core_cairo_hint_processor,
        blockifier_state: Some(build_testing_state()),
        contracts,
    };

    match runner.run_function(
        runner.find_function(config.name.as_str())?,
        &mut cairo_hint_processor,
        hints_dict,
        instructions,
        builtins,
    ) {
        Ok(result) => Ok(test_result_from_run_result(config.name.as_str(), result)),

        // CairoRunError comes from VirtualMachineError which may come from HintException that originates in the cheatcode processor
        Err(RunnerError::CairoRunError(error)) => Ok(TestResult::Failed {
            name: config.name.clone(),
            run_result: None,
            msg: Some(format!(
                "\n    {}\n",
                error.to_string().replace(" Custom Hint Error: ", "\n    ")
            )),
        }),

        Err(err) => Err(err.into()),
    }
}

pub(crate) fn skip_from_test_config(config: &TestConfig) -> TestResult {
    TestResult::Skipped {
        name: config.name.clone(),
    }
}
