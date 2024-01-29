use std::collections::HashMap;

use cairo_felt::Felt252;
use cairo_lang_casm::instructions::Instruction;
use cairo_lang_casm::{casm, casm_extend};
use cairo_lang_runner::profiling::ProfilingInfo;
use cairo_lang_runner::{Arg, RunResult, RunResultValue, RunnerError};
use cairo_lang_sierra::extensions::bitwise::BitwiseType;
use cairo_lang_sierra::extensions::core::{CoreLibfunc, CoreType};
use cairo_lang_sierra::extensions::ec::EcOpType;
use cairo_lang_sierra::extensions::enm::EnumType;
use cairo_lang_sierra::extensions::gas::GasBuiltinType;
use cairo_lang_sierra::extensions::pedersen::PedersenType;
use cairo_lang_sierra::extensions::poseidon::PoseidonType;
use cairo_lang_sierra::extensions::range_check::RangeCheckType;
use cairo_lang_sierra::extensions::segment_arena::SegmentArenaType;
use cairo_lang_sierra::extensions::starknet::syscalls::SystemType;
use cairo_lang_sierra::extensions::{ConcreteType, NamedType};
use cairo_lang_sierra::ids::{ConcreteTypeId, GenericTypeId};
use cairo_lang_sierra::program::{Function, GenericArg, StatementIdx};
use cairo_lang_sierra::program_registry::ProgramRegistry;
use cairo_lang_sierra_to_casm::compiler::CairoProgram;
use cairo_lang_sierra_to_casm::metadata::{
    calc_metadata, calc_metadata_ap_change_only, Metadata, MetadataComputationConfig, MetadataError,
};
use cairo_lang_sierra_type_size::{get_type_size_map, TypeSizeMap};
use cairo_lang_starknet::contract::ContractInfo;
use cairo_lang_utils::casts::IntoOrPanic;
use cairo_lang_utils::extract_matches;
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use cairo_lang_utils::unordered_hash_map::UnorderedHashMap;
use cairo_vm::serde::deserialize_program::{BuiltinName, HintParams};
use cairo_vm::vm::trace::trace_entry::TraceEntry;

use crate::sierra_casm_runner_gas::run_function;
use cairo_vm::vm::vm_core::VirtualMachine;
use cheatnet::runtime_extensions::forge_runtime_extension::ForgeRuntime;
use num_bigint::BigInt;
use num_traits::ToPrimitive;

// almost entirely copied from cairo_lang_runner
// fields are `pub` now
// `run_function_with_vm` uses `finalize` function to extract used resources
/// Runner enabling running a Sierra program on the vm.
pub struct SierraCasmRunner {
    /// The sierra program.
    sierra_program: cairo_lang_sierra::program::Program,
    /// Program registry for the Sierra program.
    sierra_program_registry: ProgramRegistry<CoreType, CoreLibfunc>,
    /// Program registry for the Sierra program.
    type_sizes: TypeSizeMap,
    /// The casm program matching the Sierra code.
    casm_program: CairoProgram,
    #[allow(dead_code)]
    /// Mapping from class_hash to contract info.
    starknet_contracts_info: OrderedHashMap<Felt252, ContractInfo>,
    /// Whether to run the profiler when running using this runner.
    run_profiler: bool,
}
impl SierraCasmRunner {
    pub fn new(
        sierra_program: cairo_lang_sierra::program::Program,
        metadata_config: Option<MetadataComputationConfig>,
        starknet_contracts_info: OrderedHashMap<Felt252, ContractInfo>,
        run_profiler: bool,
    ) -> Result<Self, RunnerError> {
        let gas_usage_check = metadata_config.is_some();
        let metadata = create_metadata(&sierra_program, metadata_config)?;
        let sierra_program_registry =
            ProgramRegistry::<CoreType, CoreLibfunc>::new(&sierra_program)?;
        let type_sizes = get_type_size_map(&sierra_program, &sierra_program_registry).unwrap();
        let casm_program = cairo_lang_sierra_to_casm::compiler::compile(
            &sierra_program,
            &metadata,
            gas_usage_check,
        )?;

        // Find all contracts.
        Ok(Self {
            sierra_program,
            sierra_program_registry,
            type_sizes,
            casm_program,
            starknet_contracts_info,
            run_profiler,
        })
    }

    /// Runs the vm starting from a function with custom hint processor. Function may have
    /// implicits, but no other ref params. The cost of the function is deducted from
    /// `available_gas` before the execution begins.
    ///
    /// Allows injecting Cairo `VirtualMachine`
    pub fn run_function_with_vm<'a, Bytecode>(
        &self,
        func: &Function,
        vm: &mut VirtualMachine,
        runtime: &mut ForgeRuntime,
        hints_dict: HashMap<usize, Vec<HintParams>>,
        bytecode: Bytecode,
        builtins: Vec<BuiltinName>,
    ) -> Result<RunResult, RunnerError>
    where
        Bytecode: Iterator<Item = &'a BigInt> + Clone,
    {
        let return_types = self.generic_id_and_size_from_concrete(&func.signature.ret_types);

        let (cells, ap) = run_function(
            vm,
            bytecode,
            builtins,
            cairo_lang_runner::initialize_vm,
            runtime,
            hints_dict,
        )?;
        let (results_data, gas_counter) = Self::get_results_data(&return_types, &cells, ap);
        assert!(results_data.len() <= 1);

        let value = if results_data.is_empty() {
            // No result type - no panic.
            RunResultValue::Success(vec![])
        } else {
            let (ty, values) = results_data[0].clone();
            let inner_ty = self
                .inner_type_from_panic_wrapper(&ty, func)
                .map(|it| self.type_sizes[&it]);
            Self::handle_main_return_value(inner_ty, values, &cells)
        };

        let profiling_info = if self.run_profiler {
            Some(self.collect_profiling_info(vm.get_relocated_trace().unwrap()))
        } else {
            None
        };

        Ok(RunResult {
            gas_counter,
            memory: cells,
            value,
            profiling_info,
        })
    }

    /// Collects profiling info of the current run using the trace.
    fn collect_profiling_info(&self, trace: &[TraceEntry]) -> ProfilingInfo {
        let sierra_len = self.casm_program.debug_info.sierra_statement_info.len() - 1;
        // The CASM program starts with a header of instructions to wrap the real program.
        // `real_pc_0` is the PC in the trace that points to the same CASM instruction which is in
        // the real PC=0 in the original CASM program. That is, all trace's PCs need to be
        // subtracted by `real_pc_0` to get the real PC they point to in the original CASM
        // program.
        // This is the same as the PC of the last trace entry as the header is built to have a `ret`
        // last instruction, which must be the last in the trace of any execution.
        let real_pc_0 = trace.last().unwrap().pc + 1;

        // Count the number of times each PC was executed. Note the header and footer (CASM
        // instructions added for running the program by the runner) are also counted (but are
        // later ignored).
        let pc_counts = trace.iter().fold(
            UnorderedHashMap::<usize, usize>::default(),
            |mut acc, step| {
                *acc.entry(step.pc).or_insert(0) += 1;
                acc
            },
        );

        // For each pc, find the corresponding Sierra statement, and accumulate the weight to find
        // the total weight of each Sierra statement.
        let mut sierra_statements_weights = pc_counts
            .filter(|pc, count| *count != 0 && *pc >= real_pc_0)
            .aggregate_by(
                |pc| {
                    let real_pc = pc - real_pc_0;
                    // the `-1` here can't cause an underflow as the first statement is always at
                    // offset 0, so it is always on the left side of the
                    // partition, and thus the partition index is >0.
                    let idx = self
                        .casm_program
                        .debug_info
                        .sierra_statement_info
                        .partition_point(|x| x.code_offset <= real_pc)
                        - 1;
                    StatementIdx(idx)
                },
                |x, y| x + y,
                &0,
            );
        sierra_statements_weights.remove(&StatementIdx(sierra_len));

        ProfilingInfo {
            sierra_statements_weights,
        }
    }

    /// Extract inner type if `ty` is a panic wrapper
    fn inner_type_from_panic_wrapper(
        &self,
        ty: &GenericTypeId,
        func: &Function,
    ) -> Option<ConcreteTypeId> {
        let info = func
            .signature
            .ret_types
            .iter()
            .find_map(|rt| {
                let info = self.get_info(rt);
                (info.long_id.generic_id == *ty).then_some(info)
            })
            .unwrap();

        if *ty == EnumType::ID
            && matches!(&info.long_id.generic_args[0], GenericArg::UserType(ut)
                if ut.debug_name.as_ref().unwrap().starts_with("core::panics::PanicResult::"))
        {
            return Some(extract_matches!(&info.long_id.generic_args[1], GenericArg::Type).clone());
        }
        None
    }

    /// Handling the main return value to create a `RunResultValue`.
    #[allow(clippy::cast_sign_loss)]
    pub fn handle_main_return_value(
        inner_type_size: Option<i16>,
        values: Vec<Felt252>,
        cells: &[Option<Felt252>],
    ) -> RunResultValue {
        if let Some(inner_type_size) = inner_type_size {
            // The function includes a panic wrapper.
            if values[0] == Felt252::from(0) {
                // The run resulted successfully, returning the inner value.
                let inner_ty_size = inner_type_size as usize;
                let skip_size = values.len() - inner_ty_size;
                RunResultValue::Success(values.into_iter().skip(skip_size).collect())
            } else {
                // The run resulted in a panic, returning the error data.
                let err_data_start = values[values.len() - 2].to_usize().unwrap();
                let err_data_end = values[values.len() - 1].to_usize().unwrap();
                RunResultValue::Panic(
                    cells[err_data_start..err_data_end]
                        .iter()
                        .cloned()
                        .map(Option::unwrap)
                        .collect(),
                )
            }
        } else {
            // No panic wrap - so always successful.
            RunResultValue::Success(values)
        }
    }

    /// Returns the final values and type of all `func`s returning variables.
    #[allow(clippy::cast_sign_loss)]
    pub fn get_results_data(
        return_types: &[(GenericTypeId, i16)],
        cells: &[Option<Felt252>],
        mut ap: usize,
    ) -> (Vec<(GenericTypeId, Vec<Felt252>)>, Option<Felt252>) {
        let mut results_data = vec![];
        for (ty, ty_size) in return_types.iter().rev() {
            let size = *ty_size as usize;
            let values: Vec<Felt252> = ((ap - size)..ap)
                .map(|index| cells[index].clone().unwrap())
                .collect();
            ap -= size;
            results_data.push((ty.clone(), values));
        }

        // Handling implicits.
        let mut gas_counter = None;
        results_data.retain_mut(|(ty, values)| {
            let generic_ty = ty;
            if *generic_ty == GasBuiltinType::ID {
                gas_counter = Some(values.remove(0));
                assert!(values.is_empty());
                false
            } else {
                *generic_ty != RangeCheckType::ID
                    && *generic_ty != BitwiseType::ID
                    && *generic_ty != EcOpType::ID
                    && *generic_ty != PedersenType::ID
                    && *generic_ty != PoseidonType::ID
                    && *generic_ty != SystemType::ID
                    && *generic_ty != SegmentArenaType::ID
            }
        });

        (results_data, gas_counter)
    }

    /// Finds first function ending with `name_suffix`.
    pub fn find_function(&self, name_suffix: &str) -> Result<&Function, RunnerError> {
        self.sierra_program
            .funcs
            .iter()
            .find(|f| {
                if let Some(name) = &f.id.debug_name {
                    name.ends_with(name_suffix)
                } else {
                    false
                }
            })
            .ok_or_else(|| RunnerError::MissingFunction {
                suffix: name_suffix.to_owned(),
            })
    }

    /// Converts array of `ConcreteTypeId`s into corresponding `GenericTypeId`s and their sizes
    fn generic_id_and_size_from_concrete(
        &self,
        types: &[ConcreteTypeId],
    ) -> Vec<(GenericTypeId, i16)> {
        types
            .iter()
            .map(|pt| {
                let info = self.get_info(pt);
                let generic_id = &info.long_id.generic_id;
                let size = self.type_sizes[pt];
                (generic_id.clone(), size)
            })
            .collect()
    }

    fn get_info(
        &self,
        ty: &cairo_lang_sierra::ids::ConcreteTypeId,
    ) -> &cairo_lang_sierra::extensions::types::TypeInfo {
        self.sierra_program_registry.get_type(ty).unwrap().info()
    }

    #[allow(clippy::too_many_lines)]
    pub fn create_entry_code_from_params(
        param_types: &[(GenericTypeId, i16)],
        args: &[Arg],
        initial_gas: usize,
        code_offset: usize,
    ) -> Result<(Vec<Instruction>, Vec<BuiltinName>), RunnerError> {
        let mut ctx = casm! {};
        // The builtins in the formatting expected by the runner.
        let builtins = vec![
            BuiltinName::pedersen,
            BuiltinName::range_check,
            BuiltinName::bitwise,
            BuiltinName::ec_op,
            BuiltinName::poseidon,
        ];
        // The offset [fp - i] for each of this builtins in this configuration.
        let builtin_offset: HashMap<GenericTypeId, i16> = HashMap::from([
            (PedersenType::ID, 7),
            (RangeCheckType::ID, 6),
            (BitwiseType::ID, 5),
            (EcOpType::ID, 4),
            (PoseidonType::ID, 3),
        ]);
        // Load all array args content to memory.
        let mut array_args_data = vec![];
        let mut ap_offset: i16 = 0;
        for arg in args {
            let Arg::Array(values) = arg else { continue };
            array_args_data.push(ap_offset);
            casm_extend! {ctx,
                %{ memory[ap + 0] = segments.add() %}
                ap += 1;
            }
            for (i, v) in values.iter().enumerate() {
                let arr_at: i16 = (i + 1).into_or_panic();
                casm_extend! {ctx,
                    [ap + 0] = (v.to_bigint());
                    [ap + 0] = [[ap - arr_at] + i.into_or_panic()], ap++;
                }
            }
            ap_offset += (1 + values.len()).into_or_panic::<i16>();
        }
        let mut array_args_data_iter = array_args_data.iter();
        let after_arrays_data_offset = ap_offset;
        if param_types
            .iter()
            .any(|(ty, _)| ty == &SegmentArenaType::ID)
        {
            casm_extend! {ctx,
                // SegmentArena segment.
                %{ memory[ap + 0] = segments.add() %}
                // Infos segment.
                %{ memory[ap + 1] = segments.add() %}
                ap += 2;
                [ap + 0] = 0, ap++;
                // Write Infos segment, n_constructed (0), and n_destructed (0) to the segment.
                [ap - 2] = [[ap - 3]];
                [ap - 1] = [[ap - 3] + 1];
                [ap - 1] = [[ap - 3] + 2];
            }
            ap_offset += 3;
        }
        let mut expected_arguments_size = 0;
        let mut param_index = 0;
        let mut arg_iter = args.iter().enumerate();
        for ty in param_types {
            let (generic_ty, ty_size) = ty;
            if let Some(offset) = builtin_offset.get(generic_ty) {
                casm_extend! {ctx,
                    [ap + 0] = [fp - offset], ap++;
                }
                ap_offset += 1;
            } else if generic_ty == &SystemType::ID {
                casm_extend! {ctx,
                    %{ memory[ap + 0] = segments.add() %}
                    ap += 1;
                }
                ap_offset += 1;
            } else if generic_ty == &GasBuiltinType::ID {
                casm_extend! {ctx,
                    [ap + 0] = initial_gas, ap++;
                }
                ap_offset += 1;
            } else if generic_ty == &SegmentArenaType::ID {
                let offset = -ap_offset + after_arrays_data_offset;
                casm_extend! {ctx,
                    [ap + 0] = [ap + offset] + 3, ap++;
                }
                ap_offset += 1;
            } else {
                let arg_size = *ty_size;
                let param_ap_offset_end = ap_offset + arg_size;
                expected_arguments_size += arg_size.into_or_panic::<usize>();
                while ap_offset < param_ap_offset_end {
                    let Some((arg_index, arg)) = arg_iter.next() else {
                        break;
                    };
                    match arg {
                        Arg::Value(value) => {
                            casm_extend! {ctx,
                                [ap + 0] = (value.to_bigint()), ap++;
                            }
                            ap_offset += 1;
                        }
                        Arg::Array(values) => {
                            let offset = -ap_offset + array_args_data_iter.next().unwrap();
                            casm_extend! {ctx,
                                [ap + 0] = [ap + (offset)], ap++;
                                [ap + 0] = [ap - 1] + (values.len()), ap++;
                            }
                            ap_offset += 2;
                            if ap_offset > param_ap_offset_end {
                                return Err(RunnerError::ArgumentUnaligned {
                                    param_index,
                                    arg_index,
                                });
                            }
                        }
                    }
                }
                param_index += 1;
            };
        }
        let actual_args_size = args
            .iter()
            .map(|arg| match arg {
                Arg::Value(_) => 1,
                Arg::Array(_) => 2,
            })
            .sum::<usize>();
        if expected_arguments_size != actual_args_size {
            return Err(RunnerError::ArgumentsSizeMismatch {
                expected: expected_arguments_size,
                actual: actual_args_size,
            });
        }
        let before_final_call = ctx.current_code_offset;
        let final_call_size = 3;
        let offset = final_call_size + code_offset;
        casm_extend! {ctx,
            call rel offset;
            ret;
        }
        assert_eq!(before_final_call + final_call_size, ctx.current_code_offset);
        Ok((ctx.instructions, builtins))
    }

    /// Returns the instructions to add to the beginning of the code to successfully call the main
    /// function, as well as the builtins required to execute the program.
    pub fn create_entry_code(
        &self,
        func: &Function,
        args: &[Arg],
        initial_gas: usize,
    ) -> Result<(Vec<Instruction>, Vec<BuiltinName>), RunnerError> {
        let params = self.generic_id_and_size_from_concrete(&func.signature.param_types);

        let entry_point = func.entry_point.0;
        let code_offset =
            self.casm_program.debug_info.sierra_statement_info[entry_point].code_offset;

        Self::create_entry_code_from_params(&params, args, initial_gas, code_offset)
    }

    /// Creates a list of instructions that will be appended to the program's bytecode.
    pub fn create_code_footer() -> Vec<Instruction> {
        casm! {
            // Add a `ret` instruction used in libfuncs that retrieve the current value of the `fp`
            // and `pc` registers.
            ret;
        }
        .instructions
    }

    pub fn get_casm_program(&self) -> &CairoProgram {
        &self.casm_program
    }
}

/// Creates the metadata required for a Sierra program lowering to casm.
fn create_metadata(
    sierra_program: &cairo_lang_sierra::program::Program,
    metadata_config: Option<MetadataComputationConfig>,
) -> Result<Metadata, RunnerError> {
    if let Some(metadata_config) = metadata_config {
        calc_metadata(sierra_program, metadata_config)
    } else {
        calc_metadata_ap_change_only(sierra_program)
    }
    .map_err(|err| match err {
        MetadataError::ApChangeError(err) => RunnerError::ApChangeError(err),
        MetadataError::CostError(_) => RunnerError::FailedGasCalculation,
    })
}
