use crate::gas::stats::GasStats;
use cheatnet::trace_data::{CallTrace, CallTraceNode};
use debugging::ContractsDataStore;
use starknet_api::core::{ClassHash, EntryPointSelector};
use starknet_api::execution_resources::GasVector;
use std::collections::BTreeMap;

type ContractName = String;
type Selector = String;

#[derive(Debug, Clone)]
pub struct SingleTestGasInfo {
    pub gas_used: GasVector,
    pub report_data: Option<ReportData>,
}

#[derive(Debug, Clone, Default)]
pub struct ReportData(BTreeMap<ContractName, ContractInfo>);

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

                let contract_name = get_contract_name(contracts_data, class_hash);
                let selector = get_selector(contracts_data, call.entry_point.entry_point_selector);
                let gas = call
                    .gas_report_data
                    .as_ref()
                    .expect("Gas report data must be updated after test execution")
                    .get_gas();

                report_data.update_entry(contract_name, selector, gas);
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
    fn update_entry(
        &mut self,
        contract_name: ContractName,
        selector: Selector,
        gas_used: GasVector,
    ) {
        let contract_info = self.0.entry(contract_name).or_default();

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

fn get_contract_name(contracts_data: &ContractsDataStore, class_hash: ClassHash) -> ContractName {
    contracts_data
        .get_contract_name(&class_hash)
        .map_or("forked contract", |name| name.0.as_str())
        .to_string()
}

fn get_selector(contracts_data: &ContractsDataStore, selector: EntryPointSelector) -> Selector {
    contracts_data
        .get_selector(&selector)
        .expect("`Selector` should be present")
        .0
        .clone()
}
