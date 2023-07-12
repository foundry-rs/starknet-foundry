use std::collections::HashMap;

use anyhow::Result;
use cairo_vm::serde::deserialize_program::HintParams;
use cheatable_starknet::constants::build_testing_state;
use itertools::chain;

use cairo_lang_casm::hints::Hint;
use cairo_lang_casm::instructions::Instruction;
use cairo_lang_runner::casm_run::hint_to_hint_params;
use cairo_lang_runner::{CairoHintProcessor as CoreCairoHintProcessor, RunnerError};
use cairo_lang_runner::{SierraCasmRunner, StarknetState};
use cairo_vm::vm::runners::cairo_runner::RunResources;
use test_collector::TestCase;

use crate::cheatcodes_hint_processor::CairoHintProcessor;
use crate::scarb::StarknetContractArtifacts;
use crate::test_case_summary::TestCaseSummary;

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

pub(crate) fn run_from_test_case(
    runner: &mut SierraCasmRunner,
    case: &TestCase,
    contracts: &HashMap<String, StarknetContractArtifacts>,
) -> Result<TestCaseSummary> {
    let available_gas = if let Some(available_gas) = &case.available_gas {
        Some(*available_gas)
    } else {
        Some(usize::MAX)
    };
    let func = runner.find_function(case.name.as_str())?;
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
        runner.find_function(case.name.as_str())?,
        &mut cairo_hint_processor,
        hints_dict,
        instructions,
        builtins,
    ) {
        Ok(result) => Ok(TestCaseSummary::from_run_result(result, case)),

        // CairoRunError comes from VirtualMachineError which may come from HintException that originates in the cheatcode processor
        Err(RunnerError::CairoRunError(error)) => Ok(TestCaseSummary::Failed {
            name: case.name.clone(),
            run_result: None,
            msg: Some(format!(
                "\n    {}\n",
                error.to_string().replace(" Custom Hint Error: ", "\n    ")
            )),
        }),

        Err(err) => Err(err.into()),
    }
}
