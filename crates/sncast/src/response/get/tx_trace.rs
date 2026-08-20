use crate::response::cast_message::SncastCommandMessage;
use data_transformer::{
    extract_function_from_selector, reverse_transform_input, reverse_transform_output,
};
use foundry_ui::styling::OutputBuilder;
use serde::{Serialize, Serializer};
use starknet_api::execution_utils::format_panic_data;
use starknet_rust::core::types::contract::AbiEntry;
use starknet_rust::core::types::{
    ContractClass, ExecuteInvocation, FunctionInvocation, LegacyContractAbiEntry, TransactionTrace,
};
use starknet_rust::core::utils::get_selector_from_name;
use starknet_types_core::felt::Felt;
use std::borrow::Cow;
use std::cell::Cell;
use std::collections::{HashMap, HashSet};

////////

pub struct TransactionTraceResponse {
    transaction_trace: TransactionTrace,
    output: TransactionTraceOutput,
}

enum TransactionTraceOutput {
    Human(String),
    Json { transaction_hash: Felt },
}

impl TransactionTraceResponse {
    #[must_use]
    pub fn json(transaction_hash: Felt, transaction_trace: TransactionTrace) -> Self {
        Self {
            transaction_trace,
            output: TransactionTraceOutput::Json { transaction_hash },
        }
    }

    #[must_use]
    pub fn with_contract_classes(
        transaction_hash: Felt,
        transaction_trace: TransactionTrace,
        contract_classes: HashMap<Felt, ContractClass>,
        full: bool,
    ) -> (Self, bool) {
        let (decoder, invalid_abi) = TraceDecoder::new(contract_classes);
        let human_trace = if full {
            render_full_trace(transaction_hash, &transaction_trace, Some(&decoder))
        } else {
            render_trace(transaction_hash, &transaction_trace, &decoder)
        };
        let decoding_incomplete = invalid_abi || decoder.had_decode_failure.get();

        (
            Self {
                transaction_trace,
                output: TransactionTraceOutput::Human(human_trace),
            },
            decoding_incomplete,
        )
    }

    #[must_use]
    pub fn contract_addresses_by_class_hash(
        transaction_trace: &TransactionTrace,
    ) -> HashMap<Felt, HashSet<Felt>> {
        let mut contract_addresses_by_class_hash = HashMap::new();
        for invocation in root_invocations(transaction_trace) {
            collect_contract_addresses(invocation, &mut contract_addresses_by_class_hash);
        }
        contract_addresses_by_class_hash
    }
}

impl SncastCommandMessage for TransactionTraceResponse {
    fn text(&self) -> String {
        let human_trace = match &self.output {
            TransactionTraceOutput::Human(human_trace) => Cow::Borrowed(human_trace.as_str()),
            TransactionTraceOutput::Json { transaction_hash } => Cow::Owned(render_full_trace(
                *transaction_hash,
                &self.transaction_trace,
                None,
            )),
        };

        OutputBuilder::new()
            .success_message("Transaction trace retrieved")
            .blank_line()
            .text_field(&human_trace)
            .build()
    }
}

impl Serialize for TransactionTraceResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Wrapper<'a> {
            transaction_trace: &'a TransactionTrace,
        }

        Wrapper {
            transaction_trace: &self.transaction_trace,
        }
        .serialize(serializer)
    }
}

#[derive(Default)]
struct TraceDecoder {
    sierra_abis: HashMap<Felt, Vec<AbiEntry>>,
    legacy_selectors: HashMap<(Felt, Felt), String>,
    had_decode_failure: Cell<bool>,
}

impl TraceDecoder {
    fn new(contract_classes: HashMap<Felt, ContractClass>) -> (Self, bool) {
        let mut decoder = Self::default();
        let mut invalid_abi = false;

        for (class_hash, contract_class) in contract_classes {
            match contract_class {
                ContractClass::Sierra(class) => {
                    match serde_json::from_str::<Vec<AbiEntry>>(&class.abi) {
                        Ok(abi) => {
                            decoder.sierra_abis.insert(class_hash, abi);
                        }
                        Err(_) => invalid_abi = true,
                    }
                }
                ContractClass::Legacy(class) => {
                    let Some(abi) = class.abi else {
                        invalid_abi = true;
                        continue;
                    };
                    for entry in abi {
                        if let LegacyContractAbiEntry::Function(function) = entry
                            && let Ok(selector) = get_selector_from_name(&function.name)
                        {
                            decoder
                                .legacy_selectors
                                .insert((class_hash, selector), function.name);
                        }
                    }
                }
            }
        }

        (decoder, invalid_abi)
    }

    fn selector(&self, invocation: &FunctionInvocation) -> String {
        if let Some(abi) = self.sierra_abis.get(&invocation.class_hash)
            && let Some(function) =
                extract_function_from_selector(abi, invocation.entry_point_selector)
        {
            return function.name;
        }

        let selector = self
            .legacy_selectors
            .get(&(invocation.class_hash, invocation.entry_point_selector))
            .cloned();
        if selector.is_none() && self.sierra_abis.contains_key(&invocation.class_hash) {
            self.had_decode_failure.set(true);
        }
        selector.unwrap_or_else(|| invocation.entry_point_selector.to_hex_string())
    }

    fn calldata(&self, invocation: &FunctionInvocation) -> String {
        let Some(abi) = self.sierra_abis.get(&invocation.class_hash) else {
            return format_raw_felts(&invocation.calldata);
        };

        reverse_transform_input(&invocation.calldata, abi, &invocation.entry_point_selector)
            .unwrap_or_else(|_| {
                self.had_decode_failure.set(true);
                format_raw_felts(&invocation.calldata)
            })
    }

    fn result(&self, invocation: &FunctionInvocation) -> String {
        if invocation.is_reverted {
            return format_result("panic", &format_panic_data(&invocation.result));
        }

        let result = if let Some(abi) = self.sierra_abis.get(&invocation.class_hash) {
            reverse_transform_output(&invocation.result, abi, &invocation.entry_point_selector)
                .unwrap_or_else(|_| {
                    self.had_decode_failure.set(true);
                    format_raw_felts(&invocation.result)
                })
        } else {
            format_raw_felts(&invocation.result)
        };

        format_result("success", &result)
    }
}

fn render_trace(
    transaction_hash: Felt,
    transaction_trace: &TransactionTrace,
    decoder: &TraceDecoder,
) -> String {
    let transaction_type = match transaction_trace {
        TransactionTrace::Invoke(_) => "INVOKE",
        TransactionTrace::Declare(_) => "DECLARE",
        TransactionTrace::DeployAccount(_) => "DEPLOY_ACCOUNT",
        TransactionTrace::L1Handler(_) => "L1_HANDLER",
    };
    let mut builder = OutputBuilder::new()
        .field("Type", transaction_type)
        .padded_felt_field("Transaction Hash", &transaction_hash);

    match transaction_trace {
        TransactionTrace::Invoke(trace) => {
            builder = append_optional_invocation(
                builder,
                "Validate Invocation",
                trace.validate_invocation.as_ref(),
                decoder,
            );
            builder = append_execute_invocation(
                builder,
                "Execute Invocation",
                &trace.execute_invocation,
                decoder,
            );
            builder = append_optional_invocation(
                builder,
                "Fee Transfer Invocation",
                trace.fee_transfer_invocation.as_ref(),
                decoder,
            );
        }
        TransactionTrace::Declare(trace) => {
            builder = append_optional_invocation(
                builder,
                "Validate Invocation",
                trace.validate_invocation.as_ref(),
                decoder,
            );
            builder = append_optional_invocation(
                builder,
                "Fee Transfer Invocation",
                trace.fee_transfer_invocation.as_ref(),
                decoder,
            );
        }
        TransactionTrace::DeployAccount(trace) => {
            builder = append_optional_invocation(
                builder,
                "Validate Invocation",
                trace.validate_invocation.as_ref(),
                decoder,
            );
            builder = append_invocation_section(
                builder,
                "Constructor Invocation",
                &trace.constructor_invocation,
                decoder,
            );
            builder = append_optional_invocation(
                builder,
                "Fee Transfer Invocation",
                trace.fee_transfer_invocation.as_ref(),
                decoder,
            );
        }
        TransactionTrace::L1Handler(trace) => {
            builder = append_execute_invocation(
                builder,
                "Function Invocation",
                &trace.function_invocation,
                decoder,
            );
        }
    }

    builder.build()
}

fn append_optional_invocation(
    builder: OutputBuilder,
    label: &str,
    invocation: Option<&FunctionInvocation>,
    decoder: &TraceDecoder,
) -> OutputBuilder {
    if let Some(invocation) = invocation {
        append_invocation_section(builder, label, invocation, decoder)
    } else {
        builder
    }
}

fn append_execute_invocation(
    builder: OutputBuilder,
    label: &str,
    invocation: &ExecuteInvocation,
    decoder: &TraceDecoder,
) -> OutputBuilder {
    match invocation {
        ExecuteInvocation::Success(invocation) => {
            append_invocation_section(builder, label, invocation, decoder)
        }
        ExecuteInvocation::Reverted(reverted) => append_section(builder, label, 0)
            .with_indent(2)
            .multiline_field("Revert Reason", &reverted.revert_reason),
    }
}

fn append_invocation_section(
    builder: OutputBuilder,
    label: &str,
    invocation: &FunctionInvocation,
    decoder: &TraceDecoder,
) -> OutputBuilder {
    let builder = append_section(builder, label, 0);
    append_compact_invocation(builder, invocation, decoder, 2)
}

fn append_compact_invocation(
    builder: OutputBuilder,
    invocation: &FunctionInvocation,
    decoder: &TraceDecoder,
    indent: usize,
) -> OutputBuilder {
    let mut builder = builder
        .with_indent(indent)
        .field("Entry Point Selector", &decoder.selector(invocation))
        .padded_felt_field("Contract Address", &invocation.contract_address)
        .field("Calldata", &decoder.calldata(invocation))
        .field("Result", &decoder.result(invocation));

    if !invocation.calls.is_empty() {
        builder = append_section(builder, "Calls", indent);
        for nested_call in &invocation.calls {
            builder = append_compact_invocation(builder, nested_call, decoder, indent + 2);
        }
    }

    builder
}

fn render_full_trace(
    transaction_hash: Felt,
    transaction_trace: &TransactionTrace,
    decoder: Option<&TraceDecoder>,
) -> String {
    let mut trace = serde_json::to_value(transaction_trace)
        .expect("transaction trace should serialize to JSON");
    if let Some(decoder) = decoder {
        decode_full_trace_invocations(&mut trace, transaction_trace, decoder);
    }
    let serde_json::Value::Object(mut fields) = trace else {
        unreachable!("transaction trace should serialize as an object");
    };

    let mut builder = OutputBuilder::new();
    if let Some(transaction_type) = fields.remove("type") {
        builder = append_json_field(builder, "type", &transaction_type, 0);
    }
    builder = builder.padded_felt_field("Transaction Hash", &transaction_hash);
    append_json_object(builder, &fields, 0).build()
}

fn decode_full_trace_invocations(
    value: &mut serde_json::Value,
    transaction_trace: &TransactionTrace,
    decoder: &TraceDecoder,
) {
    let serde_json::Value::Object(fields) = value else {
        unreachable!("transaction trace should serialize as an object");
    };

    match transaction_trace {
        TransactionTrace::Invoke(trace) => {
            decode_optional_invocation_field(
                fields,
                "validate_invocation",
                trace.validate_invocation.as_ref(),
                decoder,
            );
            decode_execute_invocation_field(
                fields,
                "execute_invocation",
                &trace.execute_invocation,
                decoder,
            );
            decode_optional_invocation_field(
                fields,
                "fee_transfer_invocation",
                trace.fee_transfer_invocation.as_ref(),
                decoder,
            );
        }
        TransactionTrace::Declare(trace) => {
            decode_optional_invocation_field(
                fields,
                "validate_invocation",
                trace.validate_invocation.as_ref(),
                decoder,
            );
            decode_optional_invocation_field(
                fields,
                "fee_transfer_invocation",
                trace.fee_transfer_invocation.as_ref(),
                decoder,
            );
        }
        TransactionTrace::DeployAccount(trace) => {
            decode_optional_invocation_field(
                fields,
                "validate_invocation",
                trace.validate_invocation.as_ref(),
                decoder,
            );
            decode_invocation_field(
                fields,
                "constructor_invocation",
                &trace.constructor_invocation,
                decoder,
            );
            decode_optional_invocation_field(
                fields,
                "fee_transfer_invocation",
                trace.fee_transfer_invocation.as_ref(),
                decoder,
            );
        }
        TransactionTrace::L1Handler(trace) => decode_execute_invocation_field(
            fields,
            "function_invocation",
            &trace.function_invocation,
            decoder,
        ),
    }
}

fn decode_execute_invocation_field(
    fields: &mut serde_json::Map<String, serde_json::Value>,
    name: &str,
    invocation: &ExecuteInvocation,
    decoder: &TraceDecoder,
) {
    if let ExecuteInvocation::Success(invocation) = invocation {
        decode_invocation_field(fields, name, invocation, decoder);
    }
}

fn decode_optional_invocation_field(
    fields: &mut serde_json::Map<String, serde_json::Value>,
    name: &str,
    invocation: Option<&FunctionInvocation>,
    decoder: &TraceDecoder,
) {
    if let Some(invocation) = invocation {
        decode_invocation_field(fields, name, invocation, decoder);
    }
}

fn decode_invocation_field(
    fields: &mut serde_json::Map<String, serde_json::Value>,
    name: &str,
    invocation: &FunctionInvocation,
    decoder: &TraceDecoder,
) {
    let value = fields
        .get_mut(name)
        .expect("serialized transaction trace should contain invocation field");
    decode_invocation(value, invocation, decoder);
}

fn decode_invocation(
    value: &mut serde_json::Value,
    invocation: &FunctionInvocation,
    decoder: &TraceDecoder,
) {
    let serde_json::Value::Object(fields) = value else {
        unreachable!("function invocation should serialize as an object");
    };

    fields.insert(
        "entry_point_selector".to_string(),
        serde_json::Value::String(decoder.selector(invocation)),
    );
    fields.insert(
        "calldata".to_string(),
        serde_json::Value::String(decoder.calldata(invocation)),
    );
    fields.insert(
        "result".to_string(),
        serde_json::Value::String(decoder.result(invocation)),
    );

    let serde_json::Value::Array(calls) = fields
        .get_mut("calls")
        .expect("serialized function invocation should contain calls")
    else {
        unreachable!("function invocation calls should serialize as an array");
    };
    for (value, invocation) in calls.iter_mut().zip(&invocation.calls) {
        decode_invocation(value, invocation, decoder);
    }
}

fn append_json_object(
    mut builder: OutputBuilder,
    fields: &serde_json::Map<String, serde_json::Value>,
    indent: usize,
) -> OutputBuilder {
    for (name, value) in fields {
        builder = append_json_field(builder, name, value, indent);
    }
    builder
}

fn append_json_field(
    builder: OutputBuilder,
    name: &str,
    value: &serde_json::Value,
    indent: usize,
) -> OutputBuilder {
    let label = format_field_label(name);
    match value {
        serde_json::Value::Object(fields) => {
            let builder = append_section(builder, &label, indent);
            append_json_object(builder, fields, indent + 2)
        }
        serde_json::Value::Array(values)
            if values
                .iter()
                .all(|value| !value.is_array() && !value.is_object()) =>
        {
            builder
                .with_indent(indent)
                .field(&label, &format_scalar_array(values))
        }
        serde_json::Value::Array(values) => {
            let mut builder = append_section(builder, &label, indent);
            for value in values {
                let serde_json::Value::Object(fields) = value else {
                    unreachable!("non-scalar trace array items should be objects")
                };
                builder = append_json_object(builder, fields, indent + 2);
            }
            builder
        }
        value => builder
            .with_indent(indent)
            .field(&label, &format_scalar(value)),
    }
}

fn append_section(builder: OutputBuilder, label: &str, indent: usize) -> OutputBuilder {
    builder.text_field(&format!("{}{label}", " ".repeat(indent)))
}

fn format_scalar_array(values: &[serde_json::Value]) -> String {
    let values = values.iter().map(format_scalar).collect::<Vec<_>>();
    format!("[{}]", values.join(", "))
}

fn format_scalar(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(value) => value.to_string(),
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::String(value) => value.clone(),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            unreachable!("nested JSON value should not be formatted as a scalar")
        }
    }
}

fn format_field_label(name: &str) -> String {
    name.split('_')
        .map(|word| {
            let mut characters = word.chars();
            let Some(first) = characters.next() else {
                return String::new();
            };
            first.to_uppercase().chain(characters).collect()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn root_invocations(transaction_trace: &TransactionTrace) -> Vec<&FunctionInvocation> {
    let mut invocations = Vec::new();
    match transaction_trace {
        TransactionTrace::Invoke(trace) => {
            invocations.extend(trace.validate_invocation.iter());
            if let ExecuteInvocation::Success(invocation) = &trace.execute_invocation {
                invocations.push(invocation);
            }
            invocations.extend(trace.fee_transfer_invocation.iter());
        }
        TransactionTrace::Declare(trace) => {
            invocations.extend(trace.validate_invocation.iter());
            invocations.extend(trace.fee_transfer_invocation.iter());
        }
        TransactionTrace::DeployAccount(trace) => {
            invocations.extend(trace.validate_invocation.iter());
            invocations.push(&trace.constructor_invocation);
            invocations.extend(trace.fee_transfer_invocation.iter());
        }
        TransactionTrace::L1Handler(trace) => {
            if let ExecuteInvocation::Success(invocation) = &trace.function_invocation {
                invocations.push(invocation);
            }
        }
    }
    invocations
}

fn collect_contract_addresses(
    invocation: &FunctionInvocation,
    contract_addresses_by_class_hash: &mut HashMap<Felt, HashSet<Felt>>,
) {
    contract_addresses_by_class_hash
        .entry(invocation.class_hash)
        .or_default()
        .insert(invocation.contract_address);

    for nested_call in &invocation.calls {
        collect_contract_addresses(nested_call, contract_addresses_by_class_hash);
    }
}

fn format_result(status: &str, result: &str) -> String {
    if result.is_empty() {
        status.to_string()
    } else {
        format!("{status}: {result}")
    }
}

fn format_raw_felts(felts: &[Felt]) -> String {
    felts
        .iter()
        .map(Felt::to_hex_string)
        .collect::<Vec<_>>()
        .join(", ")
}
