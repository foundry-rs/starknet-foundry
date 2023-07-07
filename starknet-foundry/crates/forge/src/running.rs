use std::collections::HashMap;

use anyhow::Result;
use blockifier::transaction::transaction_utils_for_protostar::create_state_with_trivial_validation_account;
use cairo_vm::serde::deserialize_program::HintParams;
use itertools::chain;

use cairo_lang_casm::hints::Hint;
use cairo_lang_casm::instructions::Instruction;
use cairo_lang_runner::casm_run::hint_to_hint_params;
use cairo_lang_runner::{
    CairoHintProcessor as CoreCairoHintProcessor, RunResultValue, RunnerError,
};
use cairo_lang_runner::{RunResult, SierraCasmRunner, StarknetState};
use test_collector::TestConfig;

use crate::cheatcodes_hint_processor::CairoHintProcessor;
use crate::test_results::{recover_result_data, TestResult};

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

fn map_run_result_to_test_result(name: &str, run_result: RunResult) -> TestResult {
    match run_result.value {
        RunResultValue::Success(_) => TestResult::Passed {
            name: name.to_string(),
            msg: recover_result_data(&run_result),
            run_result: Some(run_result),
        },
        RunResultValue::Panic(_) => TestResult::Failed {
            name: name.to_string(),
            msg: recover_result_data(&run_result),
            run_result: Some(run_result),
        },
    }
}

pub(crate) fn run_from_test_config(
    runner: &mut SierraCasmRunner,
    config: &TestConfig,
) -> Result<TestResult> {
    let available_gas = if let Some(available_gas) = &config.available_gas {
        Some(*available_gas)
    } else {
        Some(usize::MAX)
    };
    let func = runner.find_function(config.name.as_str())?;
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
        blockifier_state: Some(create_state_with_trivial_validation_account()),
    };
    let mut cairo_hint_processor = CairoHintProcessor {
        original_cairo_hint_processor: core_cairo_hint_processor,
        blockifier_state: Some(create_state_with_trivial_validation_account()),
    };

    match runner.run_function(
        runner.find_function(config.name.as_str())?,
        &mut cairo_hint_processor,
        hints_dict,
        instructions,
        builtins,
    ) {
        Ok(result) => Ok(map_run_result_to_test_result(config.name.as_str(), result)),

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
