use super::{
    casm::{get_assembled_program, run_assembled_program},
    entry_code::create_entry_code,
    hints::{hints_by_representation, hints_to_params},
};
use crate::running::syscall_handler::build_syscall_handler;
use crate::{forge_config::ForgeTrackedResource, package_tests::TestDetails};
use anyhow::Result;
use blockifier::execution::contract_class::TrackedResource;
use blockifier::state::{cached_state::CachedState, state_api::StateReader};
use cairo_lang_runner::Arg;
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
    let mut cached_state = CachedState::new(PhantomStateReader);
    let gas_price_vector = GasPriceVector {
        l1_gas_price: NonzeroGasPrice::new(GasPrice(2)).unwrap(),
        l1_data_gas_price: NonzeroGasPrice::new(GasPrice(2)).unwrap(),
        l2_gas_price: NonzeroGasPrice::new(GasPrice(2)).unwrap(),
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
    let (entry_code, builtins) = create_entry_code(test_details, casm_program);

    let assembled_program = get_assembled_program(casm_program, entry_code);

    let string_to_hint = hints_by_representation(&assembled_program);
    let hints_dict = hints_to_params(&assembled_program);

    let mut context = build_context(&block_info, None, &TrackedResource::from(tracked_resource));

    let syscall_handler = build_syscall_handler(
        &mut cached_state,
        &string_to_hint,
        &mut context,
        &test_details.parameter_types,
        builtins.len(),
    );

    let mut config = RawForgeConfig::default();

    let mut forge_config_runtime = ExtendedRuntime {
        extension: ForgeConfigExtension {
            config: &mut config,
        },
        extended_runtime: StarknetRuntime {
            hint_handler: syscall_handler,
            // Max gas is no longer set by `create_entry_code_from_params`
            // Instead, call to `ExternalHint::WriteRunParam` is added by it, and we need to
            // store the gas value to be read by logic handling the hint
            // TODO(#2966) we should subtract initial cost of the function from this value to be more exact.
            //  But as a workaround it should be good enough.
            user_args: vec![vec![Arg::Value(Felt::from(i64::MAX))]],
            panic_traceback: None,
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
