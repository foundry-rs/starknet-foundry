use crate::response::cast_message::SncastCommandMessage;
use crate::response::get::transaction::TransactionOutputBuilder;
use conversions::IntoConv;
use data_transformer::{
    find_entry_point_by_selector, reverse_transform_entry_point_input,
    reverse_transform_entry_point_output,
};
use foundry_ui::{Message, components::warning::WarningMessage, styling::OutputBuilder};
use itertools::Itertools;
use serde::{Serialize, Serializer, ser::Error as _};
use serde_json::Value;
use starknet_api::core::ClassHash;
use starknet_api::execution_utils::format_panic_data;
use starknet_rust::core::types::contract::AbiEntry;
use starknet_rust::core::types::{
    CallType, ContractClass, ContractStorageDiffItem, DeclareTransactionTrace, DeclaredClassItem,
    DeployAccountTransactionTrace, DeployedContractItem, EntryPointType, ExecuteInvocation,
    ExecutionResources, FunctionInvocation, InnerCallExecutionResources, InvokeTransactionTrace,
    L1HandlerTransactionTrace, LegacyContractAbiEntry, LegacyFunctionAbiType,
    MigratedCompiledClassItem, NonceUpdate, OrderedEvent, OrderedMessage, ReplacedClassItem,
    StateDiff, TransactionTrace,
};
use starknet_rust::core::utils::get_selector_from_name;
use starknet_types_core::felt::Felt;
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap, HashSet};

pub struct TransactionTraceResponse {
    trace: TransactionTrace,
    decoder: Option<TraceDecoder>,
    full: bool,
}

impl TransactionTraceResponse {
    pub fn new(trace: TransactionTrace, decoder: Option<TraceDecoder>, full: bool) -> Self {
        Self {
            trace,
            decoder,
            full,
        }
    }
}

impl Serialize for TransactionTraceResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut json = serde_json::to_value(&self.trace).map_err(S::Error::custom)?;
        if let Some(decoder) = &self.decoder {
            decode_trace_json(&self.trace, &mut json, decoder).map_err(S::Error::custom)?;

            let decoding_warnings = decoder.decoding_warnings();
            if !decoding_warnings.is_empty() {
                json["decoding_warnings"] =
                    serde_json::to_value(decoding_warnings).map_err(S::Error::custom)?;
            }
        }

        json.serialize(serializer)
    }
}

fn decode_trace_json(
    trace: &TransactionTrace,
    json: &mut Value,
    decoder: &TraceDecoder,
) -> Result<(), String> {
    match trace {
        TransactionTrace::Invoke(trace) => {
            decode_optional_invocation_json(
                trace.validate_invocation.as_ref(),
                json.get_mut("validate_invocation"),
                decoder,
            );
            decode_execute_invocation_json(
                &trace.execute_invocation,
                required_json_field(json, "execute_invocation")?,
                decoder,
            );
            decode_optional_invocation_json(
                trace.fee_transfer_invocation.as_ref(),
                json.get_mut("fee_transfer_invocation"),
                decoder,
            );
        }
        TransactionTrace::Declare(trace) => {
            decode_optional_invocation_json(
                trace.validate_invocation.as_ref(),
                json.get_mut("validate_invocation"),
                decoder,
            );
            decode_optional_invocation_json(
                trace.fee_transfer_invocation.as_ref(),
                json.get_mut("fee_transfer_invocation"),
                decoder,
            );
        }
        TransactionTrace::DeployAccount(trace) => {
            decode_optional_invocation_json(
                trace.validate_invocation.as_ref(),
                json.get_mut("validate_invocation"),
                decoder,
            );
            decode_invocation_json(
                &trace.constructor_invocation,
                required_json_field(json, "constructor_invocation")?,
                decoder,
            );
            decode_optional_invocation_json(
                trace.fee_transfer_invocation.as_ref(),
                json.get_mut("fee_transfer_invocation"),
                decoder,
            );
        }
        TransactionTrace::L1Handler(trace) => decode_execute_invocation_json(
            &trace.function_invocation,
            required_json_field(json, "function_invocation")?,
            decoder,
        ),
    }

    Ok(())
}

fn required_json_field<'a>(json: &'a mut Value, field: &str) -> Result<&'a mut Value, String> {
    json.get_mut(field)
        .ok_or_else(|| format!("missing required field `{field}` in serialized transaction trace"))
}

fn decode_optional_invocation_json(
    invocation: Option<&FunctionInvocation>,
    json: Option<&mut Value>,
    decoder: &TraceDecoder,
) {
    if let (Some(invocation), Some(json)) = (invocation, json) {
        decode_invocation_json(invocation, json, decoder);
    }
}

fn decode_execute_invocation_json(
    invocation: &ExecuteInvocation,
    json: &mut Value,
    decoder: &TraceDecoder,
) {
    if let ExecuteInvocation::Success(invocation) = invocation {
        decode_invocation_json(invocation, json, decoder);
    }
}

fn decode_invocation_json(
    invocation: &FunctionInvocation,
    json: &mut Value,
    decoder: &TraceDecoder,
) {
    json["entry_point_selector"] = Value::String(decoder.selector(invocation));
    json["calldata"] = Value::String(decoder.calldata(invocation));
    json["result"] = Value::String(decoder.result(invocation));

    if let Some(json_calls) = json.get_mut("calls").and_then(Value::as_array_mut) {
        for (call, json_call) in invocation.calls.iter().zip(json_calls) {
            decode_invocation_json(call, json_call, decoder);
        }
    }
}

impl SncastCommandMessage for TransactionTraceResponse {
    fn text(&self) -> String {
        let TransactionTraceResponse {
            trace,
            decoder,
            full,
        } = self;
        let human_text = append_trace(OutputBuilder::new(), trace, decoder.as_ref(), *full).build();
        let decoding_warnings = decoder
            .as_ref()
            .map_or_else(Vec::new, TraceDecoder::decoding_warnings);
        let builder = if decoding_warnings.is_empty() {
            OutputBuilder::new()
        } else {
            let warning_message = format_decoding_warning(&decoding_warnings);
            OutputBuilder::new()
                .text_field(&WarningMessage::new(warning_message).text())
                .blank_line()
        };

        builder
            .success_message("Transaction trace retrieved")
            .blank_line()
            .text_field(&human_text)
            .build()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
enum TraceDecodingWarning {
    MalformedAbi { class_hash: ClassHash },
    MissingAbi { class_hash: ClassHash },
    UnsupportedCairo0 { class_hash: ClassHash },
    SelectorNotFound { class_hash: ClassHash },
    CalldataDecodingFailed { class_hash: ClassHash },
    ResultDecodingFailed { class_hash: ClassHash },
}

impl TraceDecodingWarning {
    fn description(&self) -> String {
        match self {
            Self::MalformedAbi { class_hash } => {
                format!("malformed ABI for class {}", class_hash.to_hex_string())
            }
            Self::MissingAbi { class_hash } => {
                format!("missing ABI for class {}", class_hash.to_hex_string())
            }
            Self::UnsupportedCairo0 { class_hash } => format!(
                "calldata and result decoding is not supported for Cairo 0 class {}",
                class_hash.to_hex_string()
            ),
            Self::SelectorNotFound { class_hash } => format!(
                "entry point selector was not found in the ABI for class {}",
                class_hash.to_hex_string()
            ),
            Self::CalldataDecodingFailed { class_hash } => format!(
                "calldata could not be decoded with the ABI for class {}",
                class_hash.to_hex_string()
            ),
            Self::ResultDecodingFailed { class_hash } => format!(
                "result could not be decoded with the ABI for class {}",
                class_hash.to_hex_string()
            ),
        }
    }
}

#[derive(Default)]
pub struct TraceDecoder {
    sierra_abis: HashMap<ClassHash, Vec<AbiEntry>>,
    legacy_class_hashes: HashSet<ClassHash>,
    legacy_selectors: HashMap<(ClassHash, Felt, EntryPointKind), String>,
    decoding_warnings: RefCell<BTreeSet<TraceDecodingWarning>>,
}

impl TraceDecoder {
    #[must_use]
    pub fn new(contract_classes: HashMap<ClassHash, ContractClass>) -> Self {
        let mut decoder = Self::default();

        for (class_hash, contract_class) in contract_classes {
            match contract_class {
                ContractClass::Sierra(class) => {
                    if let Ok(abi) = serde_json::from_str::<Vec<AbiEntry>>(&class.abi) {
                        decoder.sierra_abis.insert(class_hash, abi);
                    } else {
                        decoder.add_warning(TraceDecodingWarning::MalformedAbi { class_hash });
                    }
                }
                ContractClass::Legacy(class) => {
                    let Some(abi) = class.abi else {
                        decoder.add_warning(TraceDecodingWarning::MissingAbi { class_hash });
                        continue;
                    };

                    decoder.add_warning(TraceDecodingWarning::UnsupportedCairo0 { class_hash });
                    decoder.legacy_class_hashes.insert(class_hash);
                    for entry in abi {
                        if let LegacyContractAbiEntry::Function(function) = entry
                            && let Ok(selector) = get_selector_from_name(&function.name)
                        {
                            decoder.legacy_selectors.insert(
                                (class_hash, selector, function.r#type.into()),
                                function.name,
                            );
                        }
                    }
                }
            }
        }

        decoder
    }

    fn selector(&self, invocation: &FunctionInvocation) -> String {
        if let Some(abi) = self.sierra_abis.get(&invocation.class_hash.into_())
            && let Some(function) = find_entry_point_by_selector(
                abi,
                invocation.entry_point_selector,
                invocation.entry_point_type,
            )
        {
            return function.name;
        }

        let selector = self
            .legacy_selectors
            .get(&(
                invocation.class_hash.into_(),
                invocation.entry_point_selector,
                invocation.entry_point_type.into(),
            ))
            .cloned();
        if selector.is_none()
            && (self
                .sierra_abis
                .contains_key(&invocation.class_hash.into_())
                || self
                    .legacy_class_hashes
                    .contains(&invocation.class_hash.into_()))
        {
            self.add_warning(TraceDecodingWarning::SelectorNotFound {
                class_hash: invocation.class_hash.into_(),
            });
        }
        selector.unwrap_or_else(|| invocation.entry_point_selector.to_hex_string())
    }

    fn calldata(&self, invocation: &FunctionInvocation) -> String {
        let Some(abi) = self.sierra_abis.get(&invocation.class_hash.into_()) else {
            return format_raw_felts(&invocation.calldata);
        };

        reverse_transform_entry_point_input(
            &invocation.calldata,
            abi,
            &invocation.entry_point_selector,
            invocation.entry_point_type,
        )
        .unwrap_or_else(|_| {
            self.add_warning(TraceDecodingWarning::CalldataDecodingFailed {
                class_hash: invocation.class_hash.into_(),
            });
            format_raw_felts(&invocation.calldata)
        })
    }

    fn result(&self, invocation: &FunctionInvocation) -> String {
        if invocation.is_reverted {
            return format_result("panic", &format_panic_data(&invocation.result));
        }

        let result = if let Some(abi) = self.sierra_abis.get(&invocation.class_hash.into_()) {
            reverse_transform_entry_point_output(
                &invocation.result,
                abi,
                &invocation.entry_point_selector,
                invocation.entry_point_type,
            )
            .unwrap_or_else(|_| {
                self.add_warning(TraceDecodingWarning::ResultDecodingFailed {
                    class_hash: invocation.class_hash.into_(),
                });
                format_raw_felts(&invocation.result)
            })
        } else {
            format_raw_felts(&invocation.result)
        };

        format_result("success", &result)
    }

    fn add_warning(&self, warning: TraceDecodingWarning) {
        self.decoding_warnings.borrow_mut().insert(warning);
    }

    fn decoding_warnings(&self) -> Vec<TraceDecodingWarning> {
        self.decoding_warnings.borrow().iter().cloned().collect()
    }
}

fn format_decoding_warning(warnings: &[TraceDecodingWarning]) -> String {
    if warnings.is_empty() {
        return String::new();
    }

    let details = warnings
        .iter()
        .map(|issue| format!("- {}", issue.description()))
        .join("\n");

    format!("Some trace data is shown as raw felts:\n{details}")
}

fn invocation_selector(invocation: &FunctionInvocation, decoder: Option<&TraceDecoder>) -> String {
    decoder.map_or_else(
        || invocation.entry_point_selector.to_hex_string(),
        |decoder| decoder.selector(invocation),
    )
}

fn invocation_calldata(invocation: &FunctionInvocation, decoder: Option<&TraceDecoder>) -> String {
    decoder.map_or_else(
        || format_raw_felts(&invocation.calldata),
        |decoder| decoder.calldata(invocation),
    )
}

fn invocation_result(invocation: &FunctionInvocation, decoder: Option<&TraceDecoder>) -> String {
    decoder.map_or_else(
        || {
            if invocation.is_reverted {
                format_result("panic", &format_panic_data(&invocation.result))
            } else {
                format_result("success", &format_raw_felts(&invocation.result))
            }
        },
        |decoder| decoder.result(invocation),
    )
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
enum EntryPointKind {
    External,
    L1Handler,
    Constructor,
}

impl From<EntryPointType> for EntryPointKind {
    fn from(value: EntryPointType) -> Self {
        match value {
            EntryPointType::External => Self::External,
            EntryPointType::L1Handler => Self::L1Handler,
            EntryPointType::Constructor => Self::Constructor,
        }
    }
}

impl From<LegacyFunctionAbiType> for EntryPointKind {
    fn from(value: LegacyFunctionAbiType) -> Self {
        match value {
            LegacyFunctionAbiType::Function => Self::External,
            LegacyFunctionAbiType::L1Handler => Self::L1Handler,
            LegacyFunctionAbiType::Constructor => Self::Constructor,
        }
    }
}

#[must_use]
fn append_trace(
    builder: OutputBuilder,
    transaction_trace: &TransactionTrace,
    decoder: Option<&TraceDecoder>,
    full: bool,
) -> OutputBuilder {
    match transaction_trace {
        TransactionTrace::Invoke(trace) => append_invoke(builder, trace, decoder, full),
        TransactionTrace::Declare(trace) => append_declare(builder, trace, decoder, full),
        TransactionTrace::DeployAccount(trace) => {
            append_deploy_account(builder, trace, decoder, full)
        }
        TransactionTrace::L1Handler(trace) => append_l1_handler(builder, trace, decoder, full),
    }
}

fn append_invoke(
    builder: OutputBuilder,
    trace: &InvokeTransactionTrace,
    decoder: Option<&TraceDecoder>,
    full: bool,
) -> OutputBuilder {
    let builder = builder.tx_type("INVOKE");
    let append_validate = |builder| {
        append_optional_invocation(
            builder,
            "Validate Invocation",
            trace.validate_invocation.as_ref(),
            decoder,
            full,
        )
    };
    let append_execute = |builder| {
        append_execute_invocation(
            builder,
            "Execute Invocation",
            &trace.execute_invocation,
            decoder,
            full,
        )
    };
    let append_fee_transfer = |builder| {
        append_optional_invocation(
            builder,
            "Fee Transfer Invocation",
            trace.fee_transfer_invocation.as_ref(),
            decoder,
            full,
        )
    };

    let builder = append_validate(builder);
    let builder = append_execute(builder);
    let builder = append_fee_transfer(builder);

    if full {
        let builder = append_execution_resources(builder, &trace.execution_resources, 0);
        append_optional_state_diff(builder, trace.state_diff.as_ref(), 0)
    } else {
        builder
    }
}

fn append_declare(
    builder: OutputBuilder,
    trace: &DeclareTransactionTrace,
    decoder: Option<&TraceDecoder>,
    full: bool,
) -> OutputBuilder {
    let builder = builder.tx_type("DECLARE");
    let append_validate = |builder| {
        append_optional_invocation(
            builder,
            "Validate Invocation",
            trace.validate_invocation.as_ref(),
            decoder,
            full,
        )
    };
    let append_fee_transfer = |builder| {
        append_optional_invocation(
            builder,
            "Fee Transfer Invocation",
            trace.fee_transfer_invocation.as_ref(),
            decoder,
            full,
        )
    };

    let builder = append_validate(builder);
    let builder = append_fee_transfer(builder);

    if full {
        let builder = append_execution_resources(builder, &trace.execution_resources, 0);
        append_optional_state_diff(builder, trace.state_diff.as_ref(), 0)
    } else {
        builder
    }
}

fn append_deploy_account(
    builder: OutputBuilder,
    trace: &DeployAccountTransactionTrace,
    decoder: Option<&TraceDecoder>,
    full: bool,
) -> OutputBuilder {
    let builder = builder.tx_type("DEPLOY_ACCOUNT");
    let append_validate = |builder| {
        append_optional_invocation(
            builder,
            "Validate Invocation",
            trace.validate_invocation.as_ref(),
            decoder,
            full,
        )
    };
    let append_constructor = |builder| {
        append_invocation_section(
            builder,
            "Constructor Invocation",
            &trace.constructor_invocation,
            decoder,
            full,
        )
    };
    let append_fee_transfer = |builder| {
        append_optional_invocation(
            builder,
            "Fee Transfer Invocation",
            trace.fee_transfer_invocation.as_ref(),
            decoder,
            full,
        )
    };

    let builder = append_validate(builder);
    let builder = append_constructor(builder);
    let builder = append_fee_transfer(builder);

    if full {
        let builder = append_execution_resources(builder, &trace.execution_resources, 0);
        append_optional_state_diff(builder, trace.state_diff.as_ref(), 0)
    } else {
        builder
    }
}

fn append_l1_handler(
    builder: OutputBuilder,
    trace: &L1HandlerTransactionTrace,
    decoder: Option<&TraceDecoder>,
    full: bool,
) -> OutputBuilder {
    let builder = builder.tx_type("L1_HANDLER");
    let append_function = |builder| {
        append_execute_invocation(
            builder,
            "Function Invocation",
            &trace.function_invocation,
            decoder,
            full,
        )
    };

    let builder = append_function(builder);

    if full {
        let builder = append_execution_resources(builder, &trace.execution_resources, 0);
        append_optional_state_diff(builder, trace.state_diff.as_ref(), 0)
    } else {
        builder
    }
}

fn append_optional_invocation(
    builder: OutputBuilder,
    label: &str,
    invocation: Option<&FunctionInvocation>,
    decoder: Option<&TraceDecoder>,
    full: bool,
) -> OutputBuilder {
    if let Some(invocation) = invocation {
        append_invocation_section(builder, label, invocation, decoder, full)
    } else {
        builder
    }
}

fn append_execute_invocation(
    builder: OutputBuilder,
    label: &str,
    invocation: &ExecuteInvocation,
    decoder: Option<&TraceDecoder>,
    full: bool,
) -> OutputBuilder {
    match invocation {
        ExecuteInvocation::Success(invocation) => {
            append_invocation_section(builder, label, invocation, decoder, full)
        }
        ExecuteInvocation::Reverted(reverted) => append_section(builder, label, 0)
            .with_indent(2)
            .field("Revert Reason", &reverted.revert_reason),
    }
}

fn append_invocation_section(
    builder: OutputBuilder,
    label: &str,
    invocation: &FunctionInvocation,
    decoder: Option<&TraceDecoder>,
    full: bool,
) -> OutputBuilder {
    let builder = append_section(builder, label, 0);
    if full {
        append_full_invocation(builder, invocation, decoder, 2)
    } else {
        append_compact_invocation(builder, invocation, decoder, 2)
    }
}

fn append_compact_invocation(
    builder: OutputBuilder,
    invocation: &FunctionInvocation,
    decoder: Option<&TraceDecoder>,
    indent: usize,
) -> OutputBuilder {
    let mut builder = builder
        .with_indent(indent)
        .field(
            "Entry Point Selector",
            &invocation_selector(invocation, decoder),
        )
        .contract_address(&invocation.contract_address)
        .field("Calldata", &invocation_calldata(invocation, decoder))
        .field("Result", &invocation_result(invocation, decoder));

    if !invocation.calls.is_empty() {
        builder = append_section(builder, "Calls", indent);
        for nested_call in &invocation.calls {
            builder = append_compact_invocation(builder, nested_call, decoder, indent + 2);
        }
    }

    builder
}

fn append_full_invocation(
    builder: OutputBuilder,
    invocation: &FunctionInvocation,
    decoder: Option<&TraceDecoder>,
    indent: usize,
) -> OutputBuilder {
    let builder = builder
        .with_indent(indent)
        .field("Call Type", format_call_type(invocation.call_type))
        .field("Calldata", &invocation_calldata(invocation, decoder))
        .padded_felt_field("Caller Address", &invocation.caller_address)
        .padded_felt_field("Class Hash", &invocation.class_hash)
        .padded_felt_field("Contract Address", &invocation.contract_address)
        .field(
            "Entry Point Selector",
            &invocation_selector(invocation, decoder),
        )
        .field(
            "Entry Point Type",
            format_entry_point_type(invocation.entry_point_type),
        );
    let builder = append_events(builder, &invocation.events, indent);
    let builder =
        append_inner_execution_resources(builder, &invocation.execution_resources, indent)
            .with_indent(indent)
            .field("Is Reverted", &invocation.is_reverted.to_string());
    let builder = append_messages(builder, &invocation.messages, indent);
    let builder = builder
        .with_indent(indent)
        .field("Result", &invocation_result(invocation, decoder));
    append_calls(builder, &invocation.calls, decoder, indent)
}

fn append_calls(
    mut builder: OutputBuilder,
    calls: &[FunctionInvocation],
    decoder: Option<&TraceDecoder>,
    indent: usize,
) -> OutputBuilder {
    if calls.is_empty() {
        return builder.with_indent(indent).field("Calls", "[]");
    }

    builder = append_section(builder, "Calls", indent);
    for call in calls {
        builder = append_full_invocation(builder, call, decoder, indent + 2);
    }
    builder
}

fn append_events(
    mut builder: OutputBuilder,
    events: &[OrderedEvent],
    indent: usize,
) -> OutputBuilder {
    if events.is_empty() {
        return builder.with_indent(indent).field("Events", "[]");
    }

    builder = append_section(builder, "Events", indent);
    for event in events {
        builder = builder
            .with_indent(indent + 2)
            .felt_list_field("Data", &event.data)
            .felt_list_field("Keys", &event.keys)
            .field("Order", &event.order.to_string());
    }
    builder
}

fn append_messages(
    mut builder: OutputBuilder,
    messages: &[OrderedMessage],
    indent: usize,
) -> OutputBuilder {
    if messages.is_empty() {
        return builder.with_indent(indent).field("Messages", "[]");
    }

    builder = append_section(builder, "Messages", indent);
    for message in messages {
        builder = builder
            .with_indent(indent + 2)
            .padded_felt_field("From Address", &message.from_address)
            .field("Order", &message.order.to_string())
            .felt_list_field("Payload", &message.payload)
            .felt_field("To Address", &message.to_address);
    }
    builder
}

fn append_execution_resources(
    builder: OutputBuilder,
    resources: &ExecutionResources,
    indent: usize,
) -> OutputBuilder {
    append_section(builder, "Execution Resources", indent)
        .with_indent(indent + 2)
        .field("L1 Data Gas", &resources.l1_data_gas.to_string())
        .field("L1 Gas", &resources.l1_gas.to_string())
        .field("L2 Gas", &resources.l2_gas.to_string())
}

fn append_inner_execution_resources(
    builder: OutputBuilder,
    resources: &InnerCallExecutionResources,
    indent: usize,
) -> OutputBuilder {
    append_section(builder, "Execution Resources", indent)
        .with_indent(indent + 2)
        .field("L1 Gas", &resources.l1_gas.to_string())
        .field("L2 Gas", &resources.l2_gas.to_string())
}

fn append_optional_state_diff(
    builder: OutputBuilder,
    state_diff: Option<&StateDiff>,
    indent: usize,
) -> OutputBuilder {
    if let Some(diff) = state_diff {
        append_state_diff(builder, diff, indent)
    } else {
        builder
    }
}

fn append_state_diff(
    builder: OutputBuilder,
    state_diff: &StateDiff,
    indent: usize,
) -> OutputBuilder {
    let builder = append_section(builder, "State Diff", indent);
    let builder = append_declared_classes(builder, &state_diff.declared_classes, indent + 2);
    let builder = append_deployed_contracts(builder, &state_diff.deployed_contracts, indent + 2);
    let builder = builder.with_indent(indent + 2).felt_list_field(
        "Deprecated Declared Classes",
        &state_diff.deprecated_declared_classes,
    );
    let builder = match state_diff.migrated_compiled_classes.as_deref() {
        Some(classes) => append_migrated_classes(builder, classes, indent + 2),
        None => builder,
    };
    let builder = append_nonces(builder, &state_diff.nonces, indent + 2);
    let builder = append_replaced_classes(builder, &state_diff.replaced_classes, indent + 2);
    append_storage_diffs(builder, &state_diff.storage_diffs, indent + 2)
}

fn append_declared_classes(
    mut builder: OutputBuilder,
    classes: &[DeclaredClassItem],
    indent: usize,
) -> OutputBuilder {
    if classes.is_empty() {
        return builder.with_indent(indent).field("Declared Classes", "[]");
    }
    builder = append_section(builder, "Declared Classes", indent);
    for class in classes {
        builder = builder
            .with_indent(indent + 2)
            .padded_felt_field("Class Hash", &class.class_hash)
            .padded_felt_field("Compiled Class Hash", &class.compiled_class_hash);
    }
    builder
}

fn append_deployed_contracts(
    mut builder: OutputBuilder,
    contracts: &[DeployedContractItem],
    indent: usize,
) -> OutputBuilder {
    if contracts.is_empty() {
        return builder
            .with_indent(indent)
            .field("Deployed Contracts", "[]");
    }
    builder = append_section(builder, "Deployed Contracts", indent);
    for contract in contracts {
        builder = builder
            .with_indent(indent + 2)
            .padded_felt_field("Address", &contract.address)
            .padded_felt_field("Class Hash", &contract.class_hash);
    }
    builder
}

fn append_migrated_classes(
    mut builder: OutputBuilder,
    classes: &[MigratedCompiledClassItem],
    indent: usize,
) -> OutputBuilder {
    if classes.is_empty() {
        return builder
            .with_indent(indent)
            .field("Migrated Compiled Classes", "[]");
    }
    builder = append_section(builder, "Migrated Compiled Classes", indent);
    for class in classes {
        builder = builder
            .with_indent(indent + 2)
            .padded_felt_field("Class Hash", &class.class_hash)
            .padded_felt_field("Compiled Class Hash", &class.compiled_class_hash);
    }
    builder
}

fn append_nonces(
    mut builder: OutputBuilder,
    nonces: &[NonceUpdate],
    indent: usize,
) -> OutputBuilder {
    if nonces.is_empty() {
        return builder.with_indent(indent).field("Nonces", "[]");
    }
    builder = append_section(builder, "Nonces", indent);
    for nonce in nonces {
        builder = builder
            .with_indent(indent + 2)
            .padded_felt_field("Contract Address", &nonce.contract_address)
            .felt_field("Nonce", &nonce.nonce);
    }
    builder
}

fn append_replaced_classes(
    mut builder: OutputBuilder,
    classes: &[ReplacedClassItem],
    indent: usize,
) -> OutputBuilder {
    if classes.is_empty() {
        return builder.with_indent(indent).field("Replaced Classes", "[]");
    }
    builder = append_section(builder, "Replaced Classes", indent);
    for class in classes {
        builder = builder
            .with_indent(indent + 2)
            .padded_felt_field("Class Hash", &class.class_hash)
            .padded_felt_field("Contract Address", &class.contract_address);
    }
    builder
}

fn append_storage_diffs(
    mut builder: OutputBuilder,
    diffs: &[ContractStorageDiffItem],
    indent: usize,
) -> OutputBuilder {
    if diffs.is_empty() {
        return builder.with_indent(indent).field("Storage Diffs", "[]");
    }
    builder = append_section(builder, "Storage Diffs", indent);
    for diff in diffs {
        builder = builder
            .with_indent(indent + 2)
            .padded_felt_field("Address", &diff.address);
        if diff.storage_entries.is_empty() {
            builder = builder
                .with_indent(indent + 2)
                .field("Storage Entries", "[]");
        } else {
            builder = append_section(builder, "Storage Entries", indent + 2);
            for entry in &diff.storage_entries {
                builder = builder
                    .with_indent(indent + 4)
                    .felt_field("Key", &entry.key)
                    .felt_field("Value", &entry.value);
            }
        }
    }
    builder
}

fn format_call_type(call_type: CallType) -> &'static str {
    match call_type {
        CallType::LibraryCall => "LIBRARY_CALL",
        CallType::Call => "CALL",
        CallType::Delegate => "DELEGATE",
    }
}

fn format_entry_point_type(entry_point_type: EntryPointType) -> &'static str {
    match entry_point_type {
        EntryPointType::External => "EXTERNAL",
        EntryPointType::L1Handler => "L1_HANDLER",
        EntryPointType::Constructor => "CONSTRUCTOR",
    }
}

fn append_section(builder: OutputBuilder, label: &str, indent: usize) -> OutputBuilder {
    builder.text_field(&format!("{}{label}", " ".repeat(indent)))
}

fn format_result(status: &str, result: &str) -> String {
    if result.is_empty() {
        status.to_string()
    } else {
        format!("{status}: {result}")
    }
}

fn format_raw_felts(felts: &[Felt]) -> String {
    felts.iter().map(Felt::to_hex_string).join(", ")
}
