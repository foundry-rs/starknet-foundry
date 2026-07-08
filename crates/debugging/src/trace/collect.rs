use crate::contracts_data_store::ContractsDataStore;
use crate::trace::types::{
    CallerAddress, ContractAddress, ContractName, ContractTrace, Event, Events, Gas, Selector,
    TestName, TraceInfo, TransformedCallResult, TransformedCalldata,
};
use crate::{Context, Trace};
use blockifier::execution::call_info::OrderedEvent;
use cheatnet::runtime_extensions::outer_call_runtime_extension::rpc::{CallFailure, CallSuccess};
use cheatnet::trace_data::{CallTrace, CallTraceNode};
use data_transformer::{
    ReverseTransformError, ReverseTransformEventError, reverse_transform_event,
    reverse_transform_input, reverse_transform_output,
};
use starknet_api::core::ClassHash;
use starknet_api::execution_utils::format_panic_data;
use starknet_rust::core::types::contract::AbiEntry;
use starknet_types_core::felt::Felt;

pub struct Collector<'a> {
    call_trace: &'a CallTrace,
    context: &'a Context,
}

impl<'a> Collector<'a> {
    /// Creates a new [`Collector`] from a given `cheatnet` [`CallTrace`], [`ContractsDataStore`] and [`Verbosity`].
    #[must_use]
    pub fn new(call_trace: &'a CallTrace, context: &'a Context) -> Collector<'a> {
        Collector {
            call_trace,
            context,
        }
    }

    pub fn collect_trace(&self, test_name: String) -> Trace {
        Trace {
            test_name: TestName(test_name),
            nested_calls: self.collect_nested_calls(),
        }
    }

    fn collect_contract_trace(&self) -> ContractTrace {
        let components = self.context.components();
        let entry_point = &self.call_trace.entry_point;
        let nested_calls = self.collect_nested_calls();
        let contract_name = self.collect_contract_name();
        let abi = self.collect_abi();

        let trace_info = TraceInfo {
            contract_name: components.contract_name(contract_name),
            entry_point_type: components.entry_point_type(entry_point.entry_point_type),
            calldata: components.calldata_lazy(|| self.collect_transformed_calldata(abi)),
            contract_address: components
                .contract_address(ContractAddress(entry_point.storage_address)),
            caller_address: components.caller_address(CallerAddress(entry_point.caller_address)),
            call_type: components.call_type(entry_point.call_type),
            nested_calls,
            call_result: components.call_result_lazy(|| self.collect_transformed_call_result(abi)),
            events: components.events_lazy(|| self.collect_events(abi)),
            gas: components.gas_lazy(|| self.collect_gas()),
        };

        ContractTrace {
            selector: self.collect_selector(),
            trace_info,
        }
    }

    fn collect_nested_calls(&self) -> Vec<ContractTrace> {
        self.call_trace
            .nested_calls
            .iter()
            .filter_map(CallTraceNode::extract_entry_point_call)
            .filter_map(|call_trace| {
                let call_trace = call_trace.borrow();

                // Filter mock calls that have empty class hashes.
                call_trace.entry_point.class_hash.is_some().then(|| {
                    Collector {
                        call_trace: &call_trace,
                        context: self.context,
                    }
                    .collect_contract_trace()
                })
            })
            .collect()
    }

    fn collect_contract_name(&self) -> ContractName {
        self.contracts_data_store()
            .get_contract_name(self.class_hash())
            .cloned()
            .unwrap_or_else(|| format_forked_contract_name(self.class_hash()))
    }

    fn collect_selector(&self) -> Selector {
        self.contracts_data_store()
            .get_selector(&self.call_trace.entry_point.entry_point_selector)
            .cloned()
            .unwrap_or_else(|| {
                Selector(format!(
                    "{:#x}",
                    self.call_trace.entry_point.entry_point_selector.0
                ))
            })
    }

    fn collect_abi(&self) -> &[AbiEntry] {
        self.contracts_data_store()
            .get_abi(self.class_hash())
            .expect("`ABI` should be present")
    }

    fn collect_transformed_calldata(&self, abi: &[AbiEntry]) -> TransformedCalldata {
        let calldata = &self.call_trace.entry_point.calldata.0;
        let selector = &self.call_trace.entry_point.entry_point_selector.0;
        let transformed = match reverse_transform_input(calldata, abi, selector) {
            Ok(s) => s,
            Err(ReverseTransformError::FunctionNotFound(_)) => format_raw_felts(calldata),
            Err(e) => panic!("Failed to decode calldata: {e}"),
        };
        TransformedCalldata(transformed)
    }

    fn collect_transformed_call_result(&self, abi: &[AbiEntry]) -> TransformedCallResult {
        let selector = &self.call_trace.entry_point.entry_point_selector.0;
        TransformedCallResult(match &self.call_trace.result {
            Ok(CallSuccess { ret_data }) => {
                let ret_data_str = match reverse_transform_output(ret_data, abi, selector) {
                    Ok(s) => s,
                    Err(ReverseTransformError::FunctionNotFound(_)) => format_raw_felts(ret_data),
                    Err(e) => panic!("Failed to decode call result: {e}"),
                };
                format_result_message("success", &ret_data_str)
            }
            Err(failure) => match failure {
                CallFailure::Recoverable { panic_data } => {
                    format_result_message("panic", &format_panic_data(panic_data))
                }
                CallFailure::Unrecoverable { msg } => {
                    format_result_message("error", &msg.to_string())
                }
            },
        })
    }

    fn collect_gas(&self) -> Gas {
        Gas(self
            .call_trace
            .gas_report_data
            .as_ref()
            .expect("Gas report data must be updated after test execution")
            .get_gas()
            .l2_gas)
    }

    fn collect_events(&self, abi: &[AbiEntry]) -> Events {
        Events(
            self.call_trace
                .events
                .iter()
                .map(|event| collect_event(event, abi))
                .collect(),
        )
    }

    fn class_hash(&self) -> &ClassHash {
        self.call_trace
            .entry_point
            .class_hash
            .as_ref()
            .expect("Entries with empty class hash are filtered in `collect_nested_calls`")
    }

    fn contracts_data_store(&self) -> &ContractsDataStore {
        self.context.contracts_data_store()
    }
}

fn format_result_message(tag: &str, message: &str) -> String {
    if message.is_empty() {
        tag.to_string()
    } else {
        format!("{tag}: {message}")
    }
}

fn format_forked_contract_name(class_hash: &ClassHash) -> ContractName {
    ContractName(format!("forked contract (class hash: {:#x})", class_hash.0))
}

fn format_raw_felts(felts: &[Felt]) -> String {
    felts
        .iter()
        .map(|felt| format!("{felt:#x}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn collect_event(event: &OrderedEvent, abi: &[AbiEntry]) -> Event {
    let keys: Vec<_> = event.event.keys.iter().map(|key| key.0).collect();
    let data = event.event.data.0.clone();

    match reverse_transform_event(&keys, &data, abi) {
        Ok(decoded) => Event::Decoded(decoded),
        Err(
            ReverseTransformEventError::EventNotFound(_)
            | ReverseTransformEventError::UnsupportedUntypedEvent(_),
        ) => Event::Raw { keys, data },
        Err(error) => panic!("Failed to decode event: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts_data_store::ContractsDataStore;
    use crate::trace::components::Components;
    use crate::trace::context::Context;
    use blockifier::execution::{
        call_info::ExtendedExecutionResources, entry_point::CallEntryPoint,
    };
    use cairo_annotations::trace_data::L1Resources;
    use cheatnet::runtime_extensions::outer_call_runtime_extension::rpc::CallSuccess;
    use cheatnet::trace_data::CallTrace;
    use starknet_api::core::{ClassHash, EntryPointSelector};
    use starknet_api::transaction::fields::Calldata;
    use starknet_api::transaction::{EventContent, EventData, EventKey};
    use starknet_rust::core::types::contract::{
        AbiEntry, AbiEvent, AbiEventEnum, EventField, EventFieldKind, TypedAbiEvent,
    };
    use starknet_rust::core::utils::get_selector_from_name;
    use starknet_types_core::felt::Felt;
    use std::collections::{HashMap, HashSet};
    use std::sync::Arc;

    fn make_call_trace(class_hash: ClassHash, selector: EntryPointSelector) -> CallTrace {
        CallTrace {
            entry_point: CallEntryPoint {
                class_hash: Some(class_hash),
                entry_point_selector: selector,
                ..Default::default()
            },
            nested_calls: vec![],
            result: Ok(CallSuccess { ret_data: vec![] }),
            used_execution_resources: ExtendedExecutionResources::default(),
            used_l1_resources: L1Resources::default(),
            used_syscalls_vm_resources: HashMap::default(),
            used_syscalls_sierra_gas: HashMap::default(),
            vm_trace: None,
            gas_consumed: 0,
            events: vec![],
            signature: vec![],
            gas_report_data: None,
        }
    }

    fn make_context(class_hash: ClassHash, abi: Vec<AbiEntry>) -> Context {
        let store =
            ContractsDataStore::for_testing(HashMap::from([(class_hash, abi)]), HashMap::new());
        Context::for_testing(store, Components::new(HashSet::new()))
    }

    fn make_ordered_event(keys: Vec<Felt>, data: Vec<Felt>) -> OrderedEvent {
        OrderedEvent {
            order: 0,
            event: EventContent {
                keys: keys.into_iter().map(EventKey).collect(),
                data: EventData(data),
            },
        }
    }

    fn typed_enum_event(name: &str, variants: &[(&str, &str, EventFieldKind)]) -> AbiEntry {
        AbiEntry::Event(AbiEvent::Typed(TypedAbiEvent::Enum(AbiEventEnum {
            name: name.to_string(),
            variants: variants
                .iter()
                .map(|(name, ty, kind)| EventField {
                    name: (*name).to_string(),
                    r#type: (*ty).to_string(),
                    kind: kind.clone(),
                })
                .collect(),
        })))
    }

    fn selector(name: &str) -> Felt {
        get_selector_from_name(name).unwrap()
    }

    #[test]
    fn collect_selector_falls_back_to_hex_when_not_in_map() {
        let class_hash = ClassHash::default();
        let felt = Felt::from_hex_unchecked("0x1234");
        let selector = EntryPointSelector(felt);

        let trace = make_call_trace(class_hash, selector);
        let context = make_context(class_hash, vec![]);
        let collector = Collector::new(&trace, &context);

        let result = collector.collect_selector();
        assert!(
            result.0.starts_with("0x"),
            "expected hex fallback, got: {}",
            result.0
        );
    }

    #[test]
    fn collect_contract_name_displays_class_hash_for_forked_contract() {
        let class_hash = ClassHash(Felt::from_hex_unchecked("0x1234"));
        let selector = EntryPointSelector(Felt::from_hex_unchecked("0x5678"));

        let trace = make_call_trace(class_hash, selector);
        let context = make_context(class_hash, vec![]);
        let collector = Collector::new(&trace, &context);

        let result = collector.collect_contract_name();
        assert_eq!(result.0, "forked contract (class hash: 0x1234)");
    }

    #[test]
    fn collect_transformed_calldata_falls_back_when_function_not_in_abi() {
        let class_hash = ClassHash::default();
        let selector = EntryPointSelector(Felt::from_hex_unchecked("0x5678"));

        let trace = make_call_trace(class_hash, selector);
        let context = make_context(class_hash, vec![]);
        let collector = Collector::new(&trace, &context);

        // Empty calldata and empty ABI → empty string fallback (no panic)
        let result = collector.collect_transformed_calldata(&[]);
        assert_eq!(result.0, "");
    }

    #[test]
    fn collect_transformed_calldata_renders_hex_when_function_not_in_abi() {
        let class_hash = ClassHash::default();
        let selector = EntryPointSelector(Felt::from_hex_unchecked("0x5678"));

        let mut trace = make_call_trace(class_hash, selector);
        trace.entry_point.calldata = Calldata(Arc::new(vec![
            Felt::from_hex_unchecked("0x1"),
            Felt::from_hex_unchecked("0x2a"),
            Felt::from_hex_unchecked("0xff"),
        ]));
        let context = make_context(class_hash, vec![]);
        let collector = Collector::new(&trace, &context);

        let result = collector.collect_transformed_calldata(&[]);
        assert_eq!(result.0, "0x1, 0x2a, 0xff");
    }

    #[test]
    fn collect_transformed_call_result_falls_back_when_function_not_in_abi() {
        let class_hash = ClassHash::default();
        let selector = EntryPointSelector(Felt::from_hex_unchecked("0x5678"));

        let trace = make_call_trace(class_hash, selector);
        let context = make_context(class_hash, vec![]);
        let collector = Collector::new(&trace, &context);

        // Empty ret_data and empty ABI → "success" (no panic)
        let result = collector.collect_transformed_call_result(&[]);
        assert_eq!(result.0, "success");
    }

    #[test]
    fn collect_transformed_call_result_renders_hex_when_function_not_in_abi() {
        let class_hash = ClassHash::default();
        let selector = EntryPointSelector(Felt::from_hex_unchecked("0x5678"));

        let mut trace = make_call_trace(class_hash, selector);
        trace.result = Ok(CallSuccess {
            ret_data: vec![
                Felt::from_hex_unchecked("0x1"),
                Felt::from_hex_unchecked("0x2a"),
                Felt::from_hex_unchecked("0xff"),
            ],
        });
        let context = make_context(class_hash, vec![]);
        let collector = Collector::new(&trace, &context);

        let result = collector.collect_transformed_call_result(&[]);
        assert_eq!(result.0, "success: 0x1, 0x2a, 0xff");
    }

    #[test]
    fn collect_event_falls_back_to_raw_when_event_not_in_abi() {
        let event = make_ordered_event(vec![Felt::from(0x123)], vec![Felt::from(0x456)]);

        let result = collect_event(&event, &[]);

        match result {
            Event::Raw { keys, data } => {
                assert_eq!(keys, vec![Felt::from(0x123)]);
                assert_eq!(data, vec![Felt::from(0x456)]);
            }
            Event::Decoded(decoded) => panic!("expected raw event fallback, got {decoded}"),
        }
    }

    #[test]
    #[should_panic(expected = "Failed to decode event: abi is invalid")]
    fn collect_event_panics_on_invalid_event_abi() {
        let abi = vec![typed_enum_event(
            "test::Event",
            &[("Missing", "test::MissingEvent", EventFieldKind::Nested)],
        )];
        let event = make_ordered_event(vec![selector("Missing")], vec![]);

        collect_event(&event, &abi);
    }
}
