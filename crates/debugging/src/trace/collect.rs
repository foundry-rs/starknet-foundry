use crate::contracts_data_store::{ContractsDataStore, NetworkLookupError};
use crate::trace::types::{
    CallerAddress, ContractAddress, ContractName, ContractTrace, Selector, TestName, TraceInfo,
    TransformedCallResult, TransformedCalldata,
};
use crate::{Trace, Verbosity};
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    CallFailure, CallResult as CheatnetCallResult,
};
use cheatnet::state::{CallTrace, CallTraceNode};
use data_transformer::{reverse_transform_input, reverse_transform_output};
use starknet::core::types::contract::AbiEntry;
use starknet_api::core::ClassHash;
use starknet_api::execution_utils::format_panic_data;

#[derive(thiserror::Error, Debug, Clone)]
pub enum CollectorError {
    #[error(transparent)]
    NetworkLookupError(#[from] NetworkLookupError),
}

pub struct Collector<'a> {
    call_trace: &'a CallTrace,
    contracts_data_store: &'a mut ContractsDataStore,
    verbosity: Verbosity,
}

impl<'a> Collector<'a> {
    /// Creates a new [`Collector`] from a given `cheatnet` [`CallTrace`], [`ContractsDataStore`] and [`Verbosity`].
    #[must_use]
    pub fn new(
        call_trace: &'a CallTrace,
        contracts_data_store: &'a mut ContractsDataStore,
        verbosity: Verbosity,
    ) -> Collector<'a> {
        Collector {
            call_trace,
            contracts_data_store,
            verbosity,
        }
    }

    pub fn collect_trace(&mut self, test_name: String) -> Result<Trace, CollectorError> {
        Ok(Trace {
            test_name: TestName(test_name),
            nested_calls: self.collect_nested_calls()?,
        })
    }

    fn collect_contract_trace(&mut self) -> Result<ContractTrace, CollectorError> {
        let verbosity = self.verbosity;
        let entry_point = &self.call_trace.entry_point;
        let nested_calls = self.collect_nested_calls()?;
        let contract_name = self.collect_contract_name();
        let abi = self.collect_abi()?.to_owned();

        let trace_info = TraceInfo {
            contract_name,
            entry_point_type: verbosity.detailed(|| entry_point.entry_point_type),
            calldata: verbosity.standard(|| self.collect_transformed_calldata(&abi)),
            contract_address: verbosity.detailed(|| ContractAddress(entry_point.storage_address)),
            caller_address: verbosity.detailed(|| CallerAddress(entry_point.caller_address)),
            call_type: verbosity.detailed(|| entry_point.call_type),
            nested_calls,
            call_result: verbosity.standard(|| self.collect_transformed_call_result(&abi)),
        };

        Ok(ContractTrace {
            selector: self.collect_selector()?,
            trace_info,
        })
    }

    fn collect_nested_calls(&mut self) -> Result<Vec<ContractTrace>, CollectorError> {
        self.call_trace
            .nested_calls
            .iter()
            .filter_map(CallTraceNode::extract_entry_point_call)
            .map(|call_trace| {
                Collector {
                    call_trace: &call_trace.borrow(),
                    contracts_data_store: self.contracts_data_store,
                    verbosity: self.verbosity,
                }
                .collect_contract_trace()
            })
            .collect::<Result<_, _>>()
    }

    fn collect_contract_name(&self) -> ContractName {
        self.contracts_data_store
            .get_contract_name(&self.class_hash())
            .cloned()
            .unwrap_or_else(|| ContractName("forked contract".to_string()))
    }

    fn collect_selector(&mut self) -> Result<Selector, CollectorError> {
        Ok(self
            .contracts_data_store
            .get_or_fetch_selector(
                &self.call_trace.entry_point.entry_point_selector,
                &self.class_hash(),
            )?
            .clone())
    }

    fn collect_abi(&mut self) -> Result<&[AbiEntry], CollectorError> {
        Ok(self
            .contracts_data_store
            .get_or_fetch_abi(&self.class_hash())?)
    }

    fn collect_transformed_calldata(&self, abi: &[AbiEntry]) -> TransformedCalldata {
        TransformedCalldata(
            reverse_transform_input(
                &self.call_trace.entry_point.calldata.0,
                abi,
                &self.call_trace.entry_point.entry_point_selector.0,
            )
            .expect("calldata should be successfully transformed"),
        )
    }

    fn collect_transformed_call_result(&self, abi: &[AbiEntry]) -> TransformedCallResult {
        TransformedCallResult(match &self.call_trace.result {
            CheatnetCallResult::Success { ret_data } => {
                let ret_data = reverse_transform_output(
                    ret_data,
                    abi,
                    &self.call_trace.entry_point.entry_point_selector.0,
                )
                .expect("call result should be successfully transformed");
                format_result_message("success", &ret_data)
            }
            CheatnetCallResult::Failure(failure) => match failure {
                CallFailure::Panic { panic_data } => {
                    format_result_message("panic", &format_panic_data(panic_data))
                }
                CallFailure::Error { msg } => format_result_message("error", &msg.to_string()),
            },
        })
    }

    fn class_hash(&self) -> ClassHash {
        *self
            .call_trace
            .entry_point
            .class_hash
            .as_ref()
            .expect("class_hash should be set in `fn execute_call_entry_point` in cheatnet")
    }
}

fn format_result_message(tag: &str, message: &str) -> String {
    if message.is_empty() {
        tag.to_string()
    } else {
        format!("{tag}: {message}")
    }
}
