use std::collections::HashMap;

use blockifier::{
    abi::constants, block_context::BlockContext, execution::entry_point::ExecutionResources,
    fee::fee_utils::calculate_l1_gas_by_vm_usage, transaction::objects::ResourcesMapping,
};
use cairo_vm::vm::runners::cairo_runner::ExecutionResources as VmExecutionResources;

#[allow(clippy::module_name_repetitions)]
#[must_use]
pub fn gas_from_execution_resources(
    block_context: &BlockContext,
    resources: &ExecutionResources,
) -> f64 {
    let resource_mapping = vm_execution_resources_to_resource_mapping(&resources.vm_resources);
    calculate_l1_gas_by_vm_usage(block_context, &resource_mapping)
        .expect("Calculating gas failed, some resources were not included.")
}

#[must_use]
fn vm_execution_resources_to_resource_mapping(
    execution_resources: &VmExecutionResources,
) -> ResourcesMapping {
    let mut map = HashMap::from([(
        constants::N_STEPS_RESOURCE.to_string(),
        execution_resources.n_steps,
    )]);
    map.extend(execution_resources.builtin_instance_counter.clone());
    ResourcesMapping(map)
}
