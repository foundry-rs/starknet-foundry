use crate::gas::stats::GasStats;
use crate::gas::utils::shorten_felt;
use cheatnet::trace_data::{CallTrace, CallTraceNode};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::{Attribute, Cell, Color, Table};
use debugging::ContractsDataStore;
use starknet_api::core::{ClassHash, EntryPointSelector};
use starknet_api::execution_resources::GasVector;
use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Display;

type ContractName = String;
type Selector = String;

#[derive(Debug, Clone, PartialOrd, PartialEq, Ord, Eq)]
pub enum ContractId {
    LocalContract(ContractName),
    ForkedContract(ClassHash),
}

#[derive(Debug, Clone)]
pub struct SingleTestGasInfo {
    pub gas_used: GasVector,
    pub report_data: Option<ReportData>,
}

#[derive(Debug, Clone, Default)]
pub struct ReportData(BTreeMap<ContractId, ContractInfo>);

#[derive(Debug, Clone, Default)]
pub struct ContractInfo {
    pub(super) gas_used: GasVector,
    pub(super) functions: BTreeMap<Selector, SelectorReportData>,
}

#[derive(Debug, Clone, Default)]
pub struct SelectorReportData {
    pub(super) gas_stats: GasStats,
    pub(super) n_calls: u64,
    pub(super) records: Vec<u64>,
}

impl SingleTestGasInfo {
    #[must_use]
    pub(crate) fn new(gas_used: GasVector) -> Self {
        Self {
            gas_used,
            report_data: None,
        }
    }

    pub(crate) fn get_with_report_data(
        self,
        trace: &CallTrace,
        contracts_data: &ContractsDataStore,
    ) -> Self {
        let mut report_data = ReportData::default();
        let mut stack = trace.nested_calls.clone();

        while let Some(call_trace_node) = stack.pop() {
            if let CallTraceNode::EntryPointCall(call) = call_trace_node {
                let call = call.borrow();
                let class_hash = call.entry_point.class_hash.expect(
                    "class_hash should be set in `fn execute_call_entry_point` in cheatnet",
                );

                let contract_id = get_contract_id(contracts_data, class_hash);
                let selector = get_selector(contracts_data, call.entry_point.entry_point_selector);
                let gas = call
                    .gas_report_data
                    .as_ref()
                    .expect("Gas report data must be updated after test execution")
                    .get_gas();

                report_data.update_entry(contract_id, selector, gas);
                stack.extend(call.nested_calls.clone());
            }
        }
        report_data.finalize();

        Self {
            gas_used: self.gas_used,
            report_data: Some(report_data),
        }
    }
}

impl ReportData {
    fn update_entry(&mut self, contract_id: ContractId, selector: Selector, gas_used: GasVector) {
        let contract_info = self.0.entry(contract_id).or_default();

        let current_gas = contract_info.gas_used;
        contract_info.gas_used = current_gas.checked_add(gas_used).unwrap_or_else(|| {
            panic!("Gas addition overflow when adding {gas_used:?} to {current_gas:?}.")
        });

        let entry = contract_info.functions.entry(selector).or_default();
        entry.records.push(gas_used.l2_gas.0);
        entry.n_calls += 1;
    }

    fn finalize(&mut self) {
        for contract_info in self.0.values_mut() {
            for gas_info in contract_info.functions.values_mut() {
                gas_info.gas_stats = GasStats::new(&gas_info.records);
            }
        }
    }
}

impl Display for ReportData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            writeln!(
                f,
                "\nNo contract gas usage data to display, no contract calls made."
            )?;
        }

        for (name, contract_info) in &self.0 {
            let table = format_table_output(contract_info, name);
            writeln!(f, "\n{table}")?;
        }
        Ok(())
    }
}

impl Display for ContractId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ContractId::LocalContract(name) => format!("{name} Contract"),
            ContractId::ForkedContract(class_hash) => {
                format!(
                    "forked contract\n(class hash: {})",
                    shorten_felt(class_hash.0)
                )
            }
        };
        write!(f, "{name}")
    }
}

fn get_contract_id(contracts_data: &ContractsDataStore, class_hash: ClassHash) -> ContractId {
    match contracts_data.get_contract_name(&class_hash) {
        Some(name) => ContractId::LocalContract(name.0.to_string()),
        None => ContractId::ForkedContract(class_hash),
    }
}

fn get_selector(contracts_data: &ContractsDataStore, selector: EntryPointSelector) -> Selector {
    contracts_data
        .get_selector(&selector)
        .expect("`Selector` should be present")
        .0
        .clone()
}

pub fn format_table_output(contract_info: &ContractInfo, contract_id: &ContractId) -> Table {
    let mut table = Table::new();
    table.apply_modifier(UTF8_ROUND_CORNERS);

    table.set_header(vec![Cell::new(contract_id.to_string()).fg(Color::Magenta)]);
    table.add_row(vec![
        Cell::new("Function Name").add_attribute(Attribute::Bold),
        Cell::new("Min").add_attribute(Attribute::Bold),
        Cell::new("Max").add_attribute(Attribute::Bold),
        Cell::new("Avg").add_attribute(Attribute::Bold),
        Cell::new("Std Dev").add_attribute(Attribute::Bold),
        Cell::new("# Calls").add_attribute(Attribute::Bold),
    ]);

    contract_info
        .functions
        .iter()
        .for_each(|(selector, report_data)| {
            table.add_row(vec![
                Cell::new(selector),
                Cell::new(report_data.gas_stats.min.to_string()),
                Cell::new(report_data.gas_stats.max.to_string()),
                Cell::new(report_data.gas_stats.mean.round().to_string()),
                Cell::new(report_data.gas_stats.std_deviation.round().to_string()),
                Cell::new(report_data.n_calls.to_string()),
            ]);
        });

    table
}
