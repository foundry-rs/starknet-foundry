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
            // OS resources and fee transfer resources are not included as they are not relevant
            // for test execution: https://github.com/foundry-rs/starknet-foundry/blob/979caf23c5d1085349e253d75682dd0e2527e321/docs/src/testing/gas-and-resource-estimation.md?plain=1#L75
            os_vm_resources: ExecutionResources::default(),
            n_reverted_steps: 0, // TODO(#3681)
            sierra_gas: self.sierra_gas,
            reverted_sierra_gas: GasAmount::ZERO, // TODO(#3681)
        }
    }

    // Put together from a few blockifier functions.
    // In a transaction (blockifier), there's only one l1_handler possible so we calculate those costs manually.
    pub fn to_message_resources(&self) -> MessageResources {
        let l2_to_l1_segment_length: usize = self
            .l2_to_l1_payload_lengths
            .iter()
            .map(|payload_length| constants::L2_TO_L1_MSG_HEADER_SIZE + payload_length)
            .sum();

        let l1_to_l2_segment_length: usize = self
            .l1_handler_payload_lengths
            .iter()
            .map(|payload_length| constants::L1_TO_L2_MSG_HEADER_SIZE + payload_length)
            .sum();

        let message_segment_length = l2_to_l1_segment_length + l1_to_l2_segment_length;

        MessageResources {
            l2_to_l1_payload_lengths: self.l2_to_l1_payload_lengths.clone(),
            message_segment_length,
            // The logic for calculating gas vector treats `l1_handler_payload_size` being `Some`
            // as indication that L1 handler was used and adds gas cost for that.
            // We need to set it to `None` if length is 0 to avoid including this extra cost.
            l1_handler_payload_size: if l1_to_l2_segment_length > 0 {
                Some(l1_to_l2_segment_length)
            } else {
                None
            },
        }
    }

    pub fn to_archival_resources(&self) -> ArchivalDataResources {
        // calldata length, signature length and code size are set to 0, because we don't include them in estimations
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

#[cfg(test)]
mod tests {
    use super::*;
    use blockifier::execution::call_info::{ChargedResources, ExecutionSummary};
    use blockifier::execution::syscalls::vm_syscall_utils::{SyscallSelector, SyscallUsage};
    use cairo_vm::types::builtin_name::BuiltinName;
    use std::collections::HashMap;

    fn create_test_resources(sierra_gas: u64, n_steps: usize) -> GasCalculationResources {
        GasCalculationResources {
            sierra_gas: GasAmount(sierra_gas),
            vm_resources: ExecutionResources {
                n_steps,
                n_memory_holes: 5,
                builtin_instance_counter: HashMap::from([(BuiltinName::range_check, 10)]),
            },
            syscalls: HashMap::new(),
            events: EventSummary::default(),
            l2_to_l1_payload_lengths: vec![],
            l1_handler_payload_lengths: vec![],
        }
    }

    #[test]
    fn format_sierra_gas_mode() {
        let resources = create_test_resources(1000, 500);
        let output = resources.format_for_display(ForgeTrackedResource::SierraGas);

        assert!(output.contains("sierra gas: 1000"));
        assert!(output.contains("steps: 500"));
    }

    #[test]
    fn format_cairo_steps_mode() {
        let resources = create_test_resources(1000, 500);
        let output = resources.format_for_display(ForgeTrackedResource::CairoSteps);

        assert!(output.contains("steps: 500"));
        assert!(output.contains("memory holes: 5"));
        assert!(!output.contains("sierra gas"));
    }

    #[test]
    fn format_with_syscalls() {
        let mut resources = create_test_resources(1000, 100);
        resources.syscalls.insert(
            SyscallSelector::CallContract,
            SyscallUsage {
                call_count: 5,
                ..Default::default()
            },
        );

        let output = resources.format_for_display(ForgeTrackedResource::SierraGas);
        assert!(output.contains("CallContract: 5"));
    }

    #[test]
    fn from_used_resources_extracts_all_fields() {
        let used_resources = UsedResources {
            syscall_usage: HashMap::new(),
            execution_summary: ExecutionSummary {
                charged_resources: ChargedResources {
                    vm_resources: ExecutionResources {
                        n_steps: 100,
                        n_memory_holes: 10,
                        builtin_instance_counter: HashMap::new(),
                    },
                    gas_consumed: GasAmount(5000),
                },
                event_summary: EventSummary {
                    n_events: 2,
                    total_event_keys: 4,
                    total_event_data_size: 100,
                },
                l2_to_l1_payload_lengths: vec![10, 20],
                ..Default::default()
            },
            l1_handler_payload_lengths: vec![5],
        };

        let resources = GasCalculationResources::from_used_resources(&used_resources);

        assert_eq!(resources.sierra_gas.0, 5000);
        assert_eq!(resources.vm_resources.n_steps, 100);
        assert_eq!(resources.events.n_events, 2);
        assert_eq!(resources.l2_to_l1_payload_lengths.len(), 2);
        assert_eq!(resources.l1_handler_payload_lengths.len(), 1);
    }

    #[test]
    fn to_computation_resources_preserves_values() {
        let resources = create_test_resources(1000, 500);
        let computation = resources.to_computation_resources();

        assert_eq!(computation.sierra_gas.0, 1000);
        assert_eq!(computation.tx_vm_resources.n_steps, 500);
    }

    #[test]
    fn to_message_resources_with_l1_handler() {
        let mut resources = create_test_resources(1000, 100);
        resources.l2_to_l1_payload_lengths = vec![10, 20];
        resources.l1_handler_payload_lengths = vec![5];

        let messages = resources.to_message_resources();

        assert_eq!(messages.l2_to_l1_payload_lengths, vec![10, 20]);
        assert!(messages.l1_handler_payload_size.is_some());
    }

    #[test]
    fn to_message_resources_without_l1_handler() {
        let resources = create_test_resources(1000, 100);
        let messages = resources.to_message_resources();

        assert!(messages.l1_handler_payload_size.is_none());
    }

    #[test]
    fn to_archival_resources_preserves_events() {
        let mut resources = create_test_resources(1000, 100);
        resources.events = EventSummary {
            n_events: 3,
            total_event_keys: 6,
            total_event_data_size: 150,
        };

        let archival = resources.to_archival_resources();

        assert_eq!(archival.event_summary.n_events, 3);
        assert_eq!(archival.event_summary.total_event_keys, 6);
    }
}
