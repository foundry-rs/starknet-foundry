use std::str::FromStr;

use crate::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use conversions::FromConv;
use data_transformer::reverse_transform_event;
use starknet_api::core::{ClassHash, ContractAddress};
use starknet_rust::core::types::contract::AbiEntry;
use starknet_types_core::felt::Felt;

pub struct FormattedEvent {
    pub event: String,
    pub contract_address: String,
    pub contract_full_module_path: Option<String>,
}

impl FormattedEvent {
    pub fn new(
        contracts_data: &ContractsData,
        contract_address: ContractAddress,
        class_hash: Option<ClassHash>,
        keys: &[Felt],
        data: &[Felt],
    ) -> Self {
        let contract_address = format!("{:#x}", Felt::from_(contract_address));

        let contract_full_module_path = class_hash
            .and_then(|class_hash| contracts_data.get_contract_module_path(&class_hash))
            .map(String::from);

        let event = class_hash
            .and_then(|class_hash| {
                let contract = contracts_data.get_contract_by_class_hash(&class_hash)?;
                let sierra = serde_json::Value::from_str(&contract.artifacts.sierra).ok()?;
                let abi = sierra.get("abi").cloned()?;
                let abi = serde_json::from_value::<Vec<AbiEntry>>(abi).ok()?;
                reverse_transform_event(keys, data, &abi).ok()
            })
            .unwrap_or_else(|| format_raw_event(keys, data));

        Self {
            event,
            contract_address,
            contract_full_module_path,
        }
    }
}

fn format_raw_event(keys: &[Felt], data: &[Felt]) -> String {
    format!(
        "{{ keys: {}, data: {} }}",
        format_felt_slice(keys),
        format_felt_slice(data)
    )
}

fn format_felt_slice(felts: &[Felt]) -> String {
    let felts = felts
        .iter()
        .map(|felt| format!("{felt:#x}"))
        .collect::<Vec<_>>()
        .join(", ");

    format!("[{felts}]")
}
