use crate::forge_config::ForgeTrackedResource;
use blockifier::abi::constants;
use blockifier::execution::call_info::EventSummary;
use blockifier::execution::syscalls::vm_syscall_utils::SyscallUsageMap;
use blockifier::fee::resources::{ArchivalDataResources, ComputationResources, MessageResources};
use cairo_vm::vm::runners::cairo_runner::ExecutionResources;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::UsedResources;
use starknet_api::execution_resources::GasAmount;

/// Resources that affect gas calculation and are displayed by `--detailed-resources`.
pub struct GasCalculationResources {
    pub sierra_gas: GasAmount,
    pub vm_resources: ExecutionResources,
    pub syscalls: SyscallUsageMap,
    pub events: EventSummary,
    pub l2_to_l1_payload_lengths: Vec<usize>,
    pub l1_handler_payload_lengths: Vec<usize>,
}

impl GasCalculationResources {
    pub fn from_used_resources(r: &UsedResources) -> Self {
        Self {
            sierra_gas: r.execution_summary.charged_resources.gas_consumed,
            vm_resources: r.execution_summary.charged_resources.vm_resources.clone(),
            syscalls: r.syscall_usage.clone(),
            events: r.execution_summary.event_summary.clone(),
            l2_to_l1_payload_lengths: r.execution_summary.l2_to_l1_payload_lengths.clone(),
            l1_handler_payload_lengths: r.l1_handler_payload_lengths.clone(),
        }
    }

    pub fn to_computation_resources(&self) -> ComputationResources {
        ComputationResources {
            tx_vm_resources: self.vm_resources.clone(),
            // OS resources (transaction type related costs) and fee transfer resources are not included
            // as they are not relevant for test execution (see documentation for details):
            // https://github.com/foundry-rs/starknet-foundry/blob/979caf23c5d1085349e253d75682dd0e2527e321/docs/src/testing/gas-and-resource-estimation.md?plain=1#L75
            os_vm_resources: ExecutionResources::default(),
            n_reverted_steps: 0, // TODO(#3681)
            sierra_gas: self.sierra_gas,
            reverted_sierra_gas: GasAmount::ZERO, // TODO(#3681)
        }
    }

    // Put together from a few blockifier functions
    // In a transaction (blockifier), there's only one l1_handler possible so we have to calculate those costs manually
    // (it's not the case in a scope of the test)
    pub fn to_message_resources(&self) -> MessageResources {
        let l2_to_l1_segment_length = self
            .l2_to_l1_payload_lengths
            .iter()
            .map(|payload_length| constants::L2_TO_L1_MSG_HEADER_SIZE + payload_length)
            .sum::<usize>();

        let l1_to_l2_segment_length = self
            .l1_handler_payload_lengths
            .iter()
            .map(|payload_length| constants::L1_TO_L2_MSG_HEADER_SIZE + payload_length)
            .sum::<usize>();

        let message_segment_length = l2_to_l1_segment_length + l1_to_l2_segment_length;

        MessageResources {
            l2_to_l1_payload_lengths: self.l2_to_l1_payload_lengths.clone(),
            message_segment_length,
            // The logic for calculating gas vector treats `l1_handler_payload_size` being `Some`
            // as indication that L1 handler was used and adds gas cost for that.
            //
            // We need to set it to `None` if length is 0 to avoid including this extra cost.
            l1_handler_payload_size: if l1_to_l2_segment_length > 0 {
                Some(l1_to_l2_segment_length)
            } else {
                None
            },
        }
    }

    pub fn to_archival_resources(&self) -> ArchivalDataResources {
        // calldata length, signature length and code size are set to 0, because
        // we don't include them in estimations
        // ref: https://github.com/foundry-rs/starknet-foundry/blob/5ce15b029135545452588c00aae580c05eb11ca8/docs/src/testing/gas-and-resource-estimation.md?plain=1#L73
        ArchivalDataResources {
            event_summary: self.events.clone(),
            calldata_length: 0,
            signature_length: 0,
            code_size: 0,
        }
    }

    pub fn format_for_display(&self, tracked_resource: ForgeTrackedResource) -> String {
        let syscalls = self.format_syscalls();
        let vm_resources_output = self.format_vm_resources();

        match tracked_resource {
            ForgeTrackedResource::CairoSteps => {
                format!(
                    "{vm_resources_output}
        syscalls: ({syscalls})
        "
                )
            }
            ForgeTrackedResource::SierraGas => {
                let mut output = format!(
                    "
        sierra gas: {}
        syscalls: ({syscalls})",
                    self.sierra_gas.0
                );

                if self.vm_resources != ExecutionResources::default() {
                    output.push_str(&vm_resources_output);
                }
                output.push('\n');

                output
            }
        }
    }

    fn format_syscalls(&self) -> String {
        let mut syscall_usage: Vec<_> = self
            .syscalls
            .iter()
            .map(|(selector, usage)| (selector, usage.call_count))
            .collect();
        // Sort syscalls by call count
        syscall_usage.sort_by(|a, b| b.1.cmp(&a.1));
        format_items(&syscall_usage)
    }

    fn format_vm_resources(&self) -> String {
        let sorted_builtins = sort_by_value(&self.vm_resources.builtin_instance_counter);
        let builtins = format_items(&sorted_builtins);

        format!(
            "
        steps: {}
        memory holes: {}
        builtins: ({})",
            self.vm_resources.n_steps, self.vm_resources.n_memory_holes, builtins
        )
    }
}

fn sort_by_value<'a, K, V, M>(map: M) -> Vec<(&'a K, &'a V)>
where
    M: IntoIterator<Item = (&'a K, &'a V)>,
    V: Ord,
{
    let mut sorted: Vec<_> = map.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    sorted
}

fn format_items<K, V>(items: &[(K, V)]) -> String
where
    K: std::fmt::Debug,
    V: std::fmt::Display,
{
    items
        .iter()
        .map(|(key, value)| format!("{key:?}: {value}"))
        .collect::<Vec<String>>()
        .join(", ")
}
