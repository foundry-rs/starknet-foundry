use crate::gas::stats::GasStats;
use cheatnet::trace_data::{CallTrace, CallTraceNode};
use debugging::ContractName as DebuggingContractName;
use debugging::ContractsDataStore;
use starknet_api::core::{ClassHash, EntryPointSelector};
use starknet_api::execution_resources::GasVector;
use std::collections::BTreeMap;

type ContractName = String;
type Selector = String;

#[derive(Debug, Clone)]
pub struct GasSingleTestInfo {
    pub gas_used: GasVector,
    pub report_data: ReportData,
}

#[derive(Debug, Clone)]
pub struct ReportData(BTreeMap<ContractName, ContractInfo>);

#[derive(Debug, Clone, Default)]
pub struct ContractInfo {
    pub gas_used: GasVector,
    pub functions: BTreeMap<Selector, SelectorReportData>,
}

#[derive(Debug, Clone, Default)]
pub struct SelectorReportData {
    pub gas_stats: GasStats,
    pub n_calls: u64,
    pub records: Vec<u64>,
}

impl GasSingleTestInfo {
    #[must_use]
    pub fn new(gas_used: GasVector) -> Self {
        Self {
            gas_used,
            report_data: ReportData(BTreeMap::new()),
        }
    }

    #[must_use]
    pub fn new_with_report(
        gas_used: GasVector,
        call_trace: &CallTrace,
        contracts_data: &ContractsDataStore,
    ) -> Self {
        Self::new(gas_used).collect_gas_data(call_trace, contracts_data)
    }

    fn collect_gas_data(mut self, trace: &CallTrace, contracts_data: &ContractsDataStore) -> Self {
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

                self.update_entry(contract_name, selector, gas);
                stack.extend(call.nested_calls.clone());
            }
        }
        self.finalize();
        self
    }

    fn update_entry(
        &mut self,
        contract_name: ContractName,
        selector: Selector,
        gas_used: GasVector,
    ) {
        let contract_info = self.report_data.0.entry(contract_name).or_default();

        if let Some(gas) = contract_info.gas_used.checked_add(gas_used) {
            contract_info.gas_used = gas;
        }

        let entry = contract_info.functions.entry(selector).or_default();
        entry.records.push(gas_used.l2_gas.0);
        entry.n_calls += 1;
    }

    fn finalize(&mut self) {
        for contract_info in self.report_data.0.values_mut() {
            for gas_info in contract_info.functions.values_mut() {
                gas_info.gas_stats = GasStats::new(&gas_info.records);
            }
        }
    }
}

fn get_contract_name(contracts_data: &ContractsDataStore, class_hash: ClassHash) -> ContractName {
    contracts_data
        .get_contract_name(&class_hash)
        .cloned()
        .unwrap_or_else(|| DebuggingContractName("forked contract".to_string()))
        .0
        .clone()
}

fn get_selector(contracts_data: &ContractsDataStore, selector: EntryPointSelector) -> Selector {
    contracts_data
        .get_selector(&selector)
        .expect("`Selector` should be present")
        .0
        .clone()
}
