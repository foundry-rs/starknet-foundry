use anyhow::Result;
use clap::Args;
use foundry_ui::OutputFormat;
use foundry_ui::components::warning::WarningMessage;
use futures::stream::{self, StreamExt};
use sncast::helpers::command::process_command_result;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::{StarknetCommandError, handle_starknet_command_error};
use sncast::response::get::tx_trace::TransactionTraceResponse;
use sncast::response::ui::UI;
use starknet_rust::core::types::{BlockId, BlockTag, ContractClass};
use starknet_rust::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet_rust::providers::{Provider, ProviderError};
use starknet_types_core::felt::Felt;
use std::collections::{HashMap, HashSet};
use std::process::ExitCode;

const MAX_CONCURRENT_CLASS_REQUESTS: usize = 8;

struct ContractClassesFetchResponse {
    classes: HashMap<Felt, ContractClass>,
    failures: Vec<ContractClassFetchFailure>,
}

struct ContractClassFetchFailure {
    class_hash: Felt,
    contract_addresses: HashSet<Felt>,
    error: ProviderError,
}

#[derive(Debug, Args)]
#[command(about = "Get the execution trace of a transaction")]
pub struct TxTrace {
    /// Hash of the transaction
    pub transaction_hash: Felt,

    /// Display all transaction trace fields
    #[arg(long)]
    pub full: bool,

    #[command(flatten)]
    pub rpc: RpcArgs,
}

pub async fn tx_trace(tx_trace: TxTrace, config: CastConfig, ui: &UI) -> Result<ExitCode> {
    let provider = tx_trace.rpc.get_provider(&config, ui).await?;

    let result = provider
        .trace_transaction(tx_trace.transaction_hash)
        .await
        .map_err(|error| StarknetCommandError::ProviderError(error.into()))
        .map_err(handle_starknet_command_error);

    let result = match result {
        Ok(trace) if ui.base_ui().output_format() == OutputFormat::Human && tx_trace.full => Ok(
            TransactionTraceResponse::full(tx_trace.transaction_hash, trace),
        ),
        Ok(trace) if ui.base_ui().output_format() == OutputFormat::Human => {
            let class_references =
                TransactionTraceResponse::contract_addresses_by_class_hash(&trace);
            let ContractClassesFetchResponse { classes, failures } =
                fetch_contract_classes(&provider, class_references).await;
            let (response, abi_decoding_incomplete) =
                TransactionTraceResponse::with_contract_classes(
                    tx_trace.transaction_hash,
                    trace,
                    classes,
                );

            if !failures.is_empty() {
                ui.print_warning(WarningMessage::new(format_class_fetch_warning(&failures)));
                ui.print_blank_line();
            }

            if abi_decoding_incomplete {
                ui.print_warning(WarningMessage::new(
                    "Some trace data could not be decoded with the fetched ABIs; raw felts are shown instead.",
                ));
                ui.print_blank_line();
            }

            Ok(response)
        }
        Ok(trace) => Ok(TransactionTraceResponse::json(
            tx_trace.transaction_hash,
            trace,
        )),
        Err(error) => Err(error),
    };

    Ok(process_command_result("get tx-trace", result, ui, None))
}

async fn fetch_contract_classes(
    provider: &JsonRpcClient<HttpTransport>,
    contract_addresses_by_class_hash: HashMap<Felt, HashSet<Felt>>,
) -> ContractClassesFetchResponse {
    let results = stream::iter(contract_addresses_by_class_hash)
        .map(|(class_hash, contract_addresses)| async move {
            match provider
                .get_class(BlockId::Tag(BlockTag::PreConfirmed), class_hash)
                .await
            {
                Ok(class) => Ok((class_hash, class)),
                Err(error) => Err(ContractClassFetchFailure {
                    class_hash,
                    contract_addresses,
                    error,
                }),
            }
        })
        .buffer_unordered(MAX_CONCURRENT_CLASS_REQUESTS)
        .collect::<Vec<_>>()
        .await;

    let mut classes = HashMap::new();
    let mut failures = Vec::new();
    for result in results {
        match result {
            Ok((class_hash, class)) => {
                classes.insert(class_hash, class);
            }
            Err(failure) => failures.push(failure),
        }
    }

    ContractClassesFetchResponse { classes, failures }
}

fn format_class_fetch_warning(failures: &[ContractClassFetchFailure]) -> String {
    let mut failures = failures.iter().collect::<Vec<_>>();
    failures.sort_unstable_by_key(|failure| failure.class_hash);

    let details = failures
        .into_iter()
        .map(|failure| {
            let mut contract_addresses = failure.contract_addresses.iter().collect::<Vec<_>>();
            contract_addresses.sort_unstable();
            let contract_addresses = contract_addresses
                .into_iter()
                .map(Felt::to_hex_string)
                .collect::<Vec<_>>()
                .join(", ");

            format!(
                "- class hash: {}, contract addresses: {} — {}",
                failure.class_hash.to_hex_string(),
                contract_addresses,
                provider_error_message(&failure.error)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "Could not fetch contract classes needed to decode the trace:\n{details}\nAffected calls are displayed as raw felts."
    )
}

fn provider_error_message(error: &ProviderError) -> String {
    match error {
        ProviderError::StarknetError(error) => error.message().to_string(),
        error => error.to_string(),
    }
}
