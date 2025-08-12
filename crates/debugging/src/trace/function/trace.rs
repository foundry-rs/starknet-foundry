use crate::contracts_data_store::ContractsDataStore;
use crate::trace::function::name::FunctionName;
use crate::trace::function::node::FunctionNode;
use crate::trace::function::stack::CallStack;
use cairo_annotations::map_pcs_to_sierra_statement_ids;
use cairo_annotations::trace_data::{CasmLevelInfo, TraceEntry};
use cairo_lang_sierra::extensions::core::{CoreConcreteLibfunc, CoreLibfunc, CoreType};
use cairo_lang_sierra::program::{GenStatement, StatementIdx};
use cairo_lang_sierra::program_registry::ProgramRegistry;
use cheatnet::state::CallTrace;
use starknet_api::core::ClassHash;

#[derive(Debug, Clone, thiserror::Error)]
pub enum FunctionTraceError {
    #[error("function trace is not supported for forked contracts")]
    ForkContract,
}
/// `FunctionTrace` represents a trace of function calls in a Sierra program.
/// It captures the structure of function calls and returns, allowing for analysis of the
/// execution flow within a contract.
#[derive(Debug, Clone)]
pub struct FunctionTrace {
    pub root: FunctionNode,
}

impl FunctionTrace {
    /// Creates a new [`FunctionTrace`] from the given [`ClassHash`], [`CallTrace`], and [`ContractsDataStore`].
    pub fn create(
        class_hash: ClassHash,
        call_trace: &CallTrace,
        contracts_data_store: &ContractsDataStore,
    ) -> Result<Self, FunctionTraceError> {
        if contracts_data_store.is_fork(&class_hash) {
            return Err(FunctionTraceError::ForkContract);
        }
        let program_artifact = contracts_data_store
            .get_program_artifact(&class_hash)
            .expect("program artifact should be present for local contracts");

        let program = &program_artifact.program;

        let sierra_program_registry = ProgramRegistry::<CoreType, CoreLibfunc>::new(program)
            .expect("failed to create Sierra program registry");

        let mut call_stack = CallStack::new();
        let mut stacks = Vec::new();

        for statement_idx in get_sierra_statements(class_hash, call_trace, contracts_data_store) {
            let function_name = FunctionName::from_program(statement_idx, program);

            let stack = call_stack.new_stack(function_name);

            stacks.push(stack.clone());

            let statement = program
                .statements
                .get(statement_idx.0)
                .expect("statement should be present");

            match statement {
                GenStatement::Invocation(invocation) => {
                    let libfunc = sierra_program_registry.get_libfunc(&invocation.libfunc_id);

                    if let Ok(CoreConcreteLibfunc::FunctionCall(_)) = &libfunc {
                        call_stack.enter_function_call(stack);
                    }
                }
                GenStatement::Return(_) => {
                    call_stack.exit_function_call();
                }
            }
        }

        Ok(build_function_trace(stacks))
    }
}

/// Retrieves vector of [`StatementIdx`] from the given [`CallTrace`].
fn get_sierra_statements(
    class_hash: ClassHash,
    call_trace: &CallTrace,
    contracts_data_store: &ContractsDataStore,
) -> Vec<StatementIdx> {
    let casm_debug_info = contracts_data_store
        .get_casm_debug_info(&class_hash)
        .expect("Cairo program debug info should be present");

    let casm_level_info = build_casm_level_info(call_trace);

    map_pcs_to_sierra_statement_ids(casm_debug_info, &casm_level_info)
        .into_iter()
        .filter_map(Option::from)
        .collect()
}

/// Builds a [`CasmLevelInfo`] from the given [`CallTrace`].
fn build_casm_level_info(call_trace: &CallTrace) -> CasmLevelInfo {
    let vm_trace = call_trace
        .vm_trace
        .as_ref()
        .expect("vm trace should be present")
        .iter()
        .map(|value| TraceEntry {
            pc: value.pc,
            ap: value.ap,
            fp: value.fp,
        })
        .collect();

    CasmLevelInfo {
        run_with_call_header: false,
        vm_trace,
    }
}

/// Builds a [`FunctionTrace`] from the given stacks of function names.
fn build_function_trace(stacks: Vec<Vec<FunctionName>>) -> FunctionTrace {
    let mut placeholder_root = FunctionNode::new(FunctionName::NonInlined("root".to_string()));

    for stack in stacks {
        placeholder_root.add_path(stack);
    }

    assert_eq!(placeholder_root.children.len(), 1);
    let real_root = placeholder_root
        .children
        .pop()
        .unwrap_or_else(|| unreachable!("assertion above should have prevented this"));

    FunctionTrace { root: real_root }
}
