use blockifier::execution::deprecated_syscalls::DeprecatedSyscallSelector::{
    StorageRead, StorageWrite,
};
use blockifier::execution::entry_point::ExecutionResources;
use cairo_vm::vm::runners::cairo_runner::ExecutionResources as VmExecutionResources;
use conversions::felt252::FromShortString;
use std::collections::HashMap;

use crate::common::{
    call_contract, deploy_contract, felt_selector_from_name, get_contracts,
    state::{create_cached_state, create_cheatnet_state},
};
use cairo_felt::Felt252;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::deploy::deploy;

// TODO (834): Verify values in this test
#[test]
fn call_resources_simple() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "HelloStarknet",
        &[],
    );

    let selector = felt_selector_from_name("increase_balance");

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[Felt252::from(123)],
    )
    .unwrap();

    assert_eq!(
        output.used_resources.execution_resources,
        ExecutionResources {
            vm_resources: VmExecutionResources {
                n_steps: 126,
                n_memory_holes: 0,
                builtin_instance_counter: HashMap::from([("range_check_builtin".to_owned(), 2)]),
            },
            syscall_counter: HashMap::from([(StorageWrite, 1), (StorageRead, 1)])
        }
    );
}

#[test]
fn deploy_resources_simple() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contracts = get_contracts();

    let contract_name = Felt252::from_short_string("HelloStarknet").unwrap();
    let class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();

    let payload = deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[]).unwrap();

    assert_eq!(payload.used_resources, ExecutionResources::default());
}

#[test]
fn deploy_resources_with_constructor() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contracts = get_contracts();

    let contract_name = Felt252::from_short_string("ConstructorSimple").unwrap();
    let class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();

    let payload = deploy(
        &mut blockifier_state,
        &mut cheatnet_state,
        &class_hash,
        &[Felt252::from(1)],
    )
    .unwrap();

    assert_eq!(
        payload.used_resources,
        ExecutionResources {
            vm_resources: VmExecutionResources {
                n_steps: 88,
                n_memory_holes: 0,
                builtin_instance_counter: HashMap::from([("range_check_builtin".to_owned(), 2)]),
            },
            syscall_counter: HashMap::from([(StorageWrite, 1)])
        }
    );
}
