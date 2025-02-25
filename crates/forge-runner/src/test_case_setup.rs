use blockifier::{
    execution::syscalls::hint_processor::SyscallHintProcessor, state::cached_state::CachedState,
};
use cairo_vm::types::builtin_name::BuiltinName;
use cheatnet::{
    constants::{get_contract_class, STRK_CLASS_HASH, STRK_CONTRACT_ADDRESS},
    data::strk_erc20_lockable::STRK_ERC_20_LOCKABLE_CASM,
    runtime_extensions::{
        call_to_blockifier_runtime_extension::rpc::UsedResources,
        forge_runtime_extension::cheatcodes::{
            declare::declare_with_contract_class, deploy::deploy_at,
        },
    },
    state::{CheatnetState, ExtendedStateReader},
};
use conversions::{felt::FromShortString, string::TryFromHexStr};
use starknet_api::core::{ClassHash, ContractAddress};
use starknet_types_core::felt::Felt;

#[must_use]
pub fn update_resources_after_predeployment(used_resources: &UsedResources) -> UsedResources {
    // Predeployment of STRK token results in modification of
    // used resources hence the need to reset/update them.

    // With predeployment
    // [PASS] deal_tests_integrationtest::test_contract::cheat_strk_balance (gas: ~737)
    //         steps: 2293
    //         memory holes: 8
    //         builtins: (range_check: 91, pedersen: 16)
    //         syscalls: (StorageWrite: 13, StorageRead: 8, EmitEvent: 4, GetExecutionInfo: 2)

    // Empty test case (without predeployment)
    // [PASS] deal_tests_integrationtest::test_contract::cheat_strk_balance (gas: ~1)
    //         steps: 68
    //         memory holes: 8
    //         builtins: (range_check: 3)
    //         syscalls: ()

    let mut used_resources = used_resources.clone();

    used_resources.execution_resources.n_steps -= 2225;

    used_resources.syscall_counter.clear();
    used_resources
        .execution_resources
        .builtin_instance_counter
        .clear();
    used_resources
        .execution_resources
        .builtin_instance_counter
        .clear();
    used_resources
        .execution_resources
        .builtin_instance_counter
        .insert(BuiltinName::range_check, 3);
    used_resources
}

pub fn declare_strk(cached_state: &mut CachedState<ExtendedStateReader>) {
    declare_with_contract_class(
        cached_state,
        get_contract_class(STRK_ERC_20_LOCKABLE_CASM),
        ClassHash::try_from_hex_str(STRK_CLASS_HASH).unwrap(),
    )
    .expect("Failed to declare STRK contract");
}

pub fn predeploy_strk(
    syscall_handler: &mut SyscallHintProcessor,
    cheatnet_state: &mut CheatnetState,
) {
    let calldata = vec![
        // name
        Felt::from_short_string("STRK").unwrap(),
        // symbol
        Felt::from_short_string("STRK").unwrap(),
        // decimals
        18.into(),
        // initial_supply low
        100_000_000.into(),
        // initial_supply high
        0.into(),
        // recipient
        123.into(),
        // permitted_minter
        123.into(),
        // provisional_governance_admin
        123.into(),
        // upgrade_delay
        0.into(),
    ];
    let class_hash = ClassHash::try_from_hex_str(STRK_CLASS_HASH).unwrap();
    let contract_address =
        ContractAddress::try_from(Felt::try_from_hex_str(STRK_CONTRACT_ADDRESS).unwrap()).unwrap();
    let deploy_result = deploy_at(
        syscall_handler,
        cheatnet_state,
        &class_hash,
        &calldata,
        contract_address,
        true,
    );

    match deploy_result {
        Ok((..)) => {}
        Err(err) => {
            panic!("Failed to deploy STRK contract: {err:?}");
        }
    }
}
