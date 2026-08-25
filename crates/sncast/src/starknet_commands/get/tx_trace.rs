use anyhow::Result;
use clap::Args;
use foundry_ui::components::warning::WarningMessage;
use futures::stream::{self, StreamExt};
use itertools::Itertools;
use sncast::helpers::command::process_command_result;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::{StarknetCommandError, handle_starknet_command_error};
use sncast::response::get::tx_trace::{
    ContractClassFetchFailure, ContractClassesFetchResponse, TraceDecoder,
    TransactionTraceResponse, contract_addresses_by_class_hash,
};
use sncast::response::ui::UI;
use starknet_rust::core::types::{BlockId, BlockTag};
use starknet_rust::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet_rust::providers::{Provider, ProviderError};
use starknet_types_core::felt::Felt;
use std::collections::{HashMap, HashSet};
use std::process::ExitCode;

const MAX_CONCURRENT_CLASS_REQUESTS: usize = 4;

#[derive(Debug, Args)]
#[command(about = "Get the execution trace of a transaction")]
pub struct TxTrace {
    /// Hash of the transaction
    pub transaction_hash: Felt,

    /// Display all transaction trace fields
    #[arg(long, conflicts_with = "json")]
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
        Ok(trace) => {
            let contract_classes_fetch_response =
                fetch_contract_classes(&provider, contract_addresses_by_class_hash(&trace)).await;

            let ContractClassesFetchResponse { classes, failures } =
                contract_classes_fetch_response;

            if !failures.is_empty() {
                ui.print_warning(WarningMessage::new(format_class_fetch_warning(&failures)));
                ui.print_blank_line();
            }

            let decoder = TraceDecoder::new(classes);
            Ok(TransactionTraceResponse::new(trace, decoder, tx_trace.full))
        }
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

    let (classes, failures): (HashMap<_, _>, Vec<_>) = results.into_iter().partition_result();

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
                .join(", ");

            format!(
                "- class hash: {}, contract addresses: {} — {}",
                failure.class_hash.to_hex_string(),
                contract_addresses,
                provider_error_message(&failure.error)
            )
        })
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
