use super::{
    VmExecutionContext, build_test_call_and_entry_point, hints::hints_by_representation,
    initialize_execution_context,
};
use crate::running::copied_code::{finalize_execution, prepare_call_arguments, run_entry_point};
use crate::{forge_config::ForgeTrackedResource, package_tests::TestDetails};
use anyhow::Result;
use blockifier::execution::contract_class::TrackedResource;
use blockifier::state::{cached_state::CachedState, state_api::StateReader};
use cheatnet::runtime_extensions::forge_config_extension::{
    ForgeConfigExtension, config::RawForgeConfig,
};
use runtime::{ExtendedRuntime, StarknetRuntime, starknet::context::build_context};
use starknet_api::block::{
    BlockInfo, BlockNumber, BlockTimestamp, GasPrice, GasPriceVector, GasPrices, NonzeroGasPrice,
};
use starknet_types_core::felt::Felt;
use std::default::Default;
use universal_sierra_compiler_api::AssembledProgramWithDebugInfo;

struct PhantomStateReader;

impl StateReader for PhantomStateReader {
    fn get_storage_at(
        &self,
        _contract_address: starknet_api::core::ContractAddress,
        _key: starknet_api::state::StorageKey,
    ) -> blockifier::state::state_api::StateResult<Felt> {
        unreachable!()
    }
    fn get_nonce_at(
        &self,
        _contract_address: starknet_api::core::ContractAddress,
    ) -> blockifier::state::state_api::StateResult<starknet_api::core::Nonce> {
        unreachable!()
    }
    fn get_class_hash_at(
        &self,
        _contract_address: starknet_api::core::ContractAddress,
    ) -> blockifier::state::state_api::StateResult<starknet_api::core::ClassHash> {
        unreachable!()
    }
    fn get_compiled_class(
        &self,
        _class_hash: starknet_api::core::ClassHash,
    ) -> blockifier::state::state_api::StateResult<
        blockifier::execution::contract_class::RunnableCompiledClass,
    > {
        unreachable!()
    }
    fn get_compiled_class_hash(
        &self,
        _class_hash: starknet_api::core::ClassHash,
    ) -> blockifier::state::state_api::StateResult<starknet_api::core::CompiledClassHash> {
        unreachable!()
    }
}

pub fn run_config_pass(
    test_details: &TestDetails,
    casm_program: &AssembledProgramWithDebugInfo,
    tracked_resource: &ForgeTrackedResource,
) -> Result<RawForgeConfig> {
    let program = test_details.try_into_program(casm_program)?;
    let (call, entry_point) = build_test_call_and_entry_point(test_details, casm_program, &program);

    let mut cached_state = CachedState::new(PhantomStateReader);
    let gas_price_vector = GasPriceVector {
        l1_gas_price: NonzeroGasPrice::new(GasPrice(2))?,
        l1_data_gas_price: NonzeroGasPrice::new(GasPrice(2))?,
        l2_gas_price: NonzeroGasPrice::new(GasPrice(2))?,
    };

    let block_info = BlockInfo {
        block_number: BlockNumber(0),
        block_timestamp: BlockTimestamp(0),
        gas_prices: GasPrices {
            eth_gas_prices: gas_price_vector.clone(),
            strk_gas_prices: gas_price_vector,
        },
        sequencer_address: 0_u8.into(),
        use_kzg_da: true,
    };
    let mut context = build_context(&block_info, None, &TrackedResource::from(tracked_resource));

    let hints = hints_by_representation(&casm_program.assembled_cairo_program);
    let VmExecutionContext {
        mut runner,
        syscall_handler,
        initial_syscall_ptr,
        program_extra_data_length,
    } = initialize_execution_context(
        call.clone(),
        &hints,
        &program,
        &mut cached_state,
        &mut context,
    )?;

    let mut config = RawForgeConfig::default();

    let mut forge_config_runtime = ExtendedRuntime {
        extension: ForgeConfigExtension {
            config: &mut config,
        },
        extended_runtime: StarknetRuntime {
            hint_handler: syscall_handler,
            user_args: vec![],
        },
    };

    let tracked_resource = TrackedResource::from(tracked_resource);
    let entry_point_initial_budget = forge_config_runtime
        .extended_runtime
        .hint_handler
        .base
        .context
        .gas_costs()
        .base
        .entry_point_initial_budget;
    let args = prepare_call_arguments(
        &forge_config_runtime
            .extended_runtime
            .hint_handler
            .base
            .call
            .clone()
            .into(),
        &mut runner,
        initial_syscall_ptr,
        &mut forge_config_runtime
            .extended_runtime
            .hint_handler
            .read_only_segments,
        &entry_point,
        entry_point_initial_budget,
    )?;
    let n_total_args = args.len();

    let bytecode_length = program.data_len();
    let program_segment_size = bytecode_length + program_extra_data_length;

    run_entry_point(
        &mut runner,
        &mut forge_config_runtime,
        entry_point,
        args,
        program_segment_size,
    )?;

    finalize_execution(
        &mut runner,
        &mut forge_config_runtime.extended_runtime.hint_handler,
        n_total_args,
        program_extra_data_length,
        tracked_resource,
    )?;

    Ok(config)
}
