use super::{
    casm::{get_assembled_program, run_assembled_program},
    entry_code::create_entry_code,
    hints::{hints_by_representation, hints_to_params},
};
use crate::{package_tests::TestDetails, running::build_syscall_handler};
use anyhow::Result;
use blockifier::{
    blockifier::block::{BlockInfo, GasPrices},
    state::{cached_state::CachedState, state_api::StateReader},
};
use cairo_vm::vm::runners::cairo_runner::ExecutionResources;
use cheatnet::runtime_extensions::forge_config_extension::{
    ForgeConfigExtension, config::RawForgeConfig,
};
use runtime::{ExtendedRuntime, StarknetRuntime, starknet::context::build_context};
use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_types_core::felt::Felt;
use std::{default::Default, num::NonZeroU128};
use universal_sierra_compiler_api::AssembledProgramWithDebugInfo;
struct PhantomStateReader;

impl StateReader for PhantomStateReader {
    fn get_class_hash_at(
        &self,
        _contract_address: starknet_api::core::ContractAddress,
    ) -> blockifier::state::state_api::StateResult<starknet_api::core::ClassHash> {
        unreachable!()
    }
    fn get_compiled_class_hash(
        &self,
        _class_hash: starknet_api::core::ClassHash,
    ) -> blockifier::state::state_api::StateResult<starknet_api::core::CompiledClassHash> {
        unreachable!()
    }
    fn get_compiled_contract_class(
        &self,
        _class_hash: starknet_api::core::ClassHash,
    ) -> blockifier::state::state_api::StateResult<
        blockifier::execution::contract_class::ContractClass,
    > {
        unreachable!()
    }
    fn get_nonce_at(
        &self,
        _contract_address: starknet_api::core::ContractAddress,
    ) -> blockifier::state::state_api::StateResult<starknet_api::core::Nonce> {
        unreachable!()
    }
    fn get_storage_at(
        &self,
        _contract_address: starknet_api::core::ContractAddress,
        _key: starknet_api::state::StorageKey,
    ) -> blockifier::state::state_api::StateResult<Felt> {
        unreachable!()
    }
}

#[allow(clippy::too_many_lines)]
pub fn run_config_pass(
    args: Vec<Felt>,
    test_details: &TestDetails,
    casm_program: &AssembledProgramWithDebugInfo,
) -> Result<RawForgeConfig> {
    let mut cached_state = CachedState::new(PhantomStateReader);
    let block_info = BlockInfo {
        block_number: BlockNumber(0),
        block_timestamp: BlockTimestamp(0),
        gas_prices: GasPrices {
            eth_l1_data_gas_price: NonZeroU128::new(2).unwrap(),
            eth_l1_gas_price: NonZeroU128::new(2).unwrap(),
            strk_l1_data_gas_price: NonZeroU128::new(2).unwrap(),
            strk_l1_gas_price: NonZeroU128::new(2).unwrap(),
        },
        sequencer_address: 0_u8.into(),
        use_kzg_da: true,
    };
    let (entry_code, builtins) = create_entry_code(args, test_details, casm_program);

    let assembled_program = get_assembled_program(casm_program, entry_code);

    let string_to_hint = hints_by_representation(&assembled_program);
    let hints_dict = hints_to_params(&assembled_program);

    let mut context = build_context(&block_info, None);

    let mut execution_resources = ExecutionResources::default();

    let syscall_handler = build_syscall_handler(
        &mut cached_state,
        &string_to_hint,
        &mut execution_resources,
        &mut context,
        &test_details.parameter_types,
    );

    let mut config = RawForgeConfig::default();

    let mut forge_config_runtime = ExtendedRuntime {
        extension: ForgeConfigExtension {
            config: &mut config,
        },
        extended_runtime: StarknetRuntime {
            hint_handler: syscall_handler,
        },
    };

    run_assembled_program(
        &assembled_program,
        builtins,
        hints_dict,
        &mut forge_config_runtime,
    )?;

    Ok(config)
}
