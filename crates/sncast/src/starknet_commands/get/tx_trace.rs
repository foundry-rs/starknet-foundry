use anyhow::{Result, bail};
use clap::Args;
use foundry_ui::{OutputFormat, components::warning::WarningMessage};
use futures::stream::{self, StreamExt};
use itertools::Itertools;
use sncast::helpers::command::process_command_result;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::{StarknetCommandError, handle_starknet_command_error};
use sncast::response::get::tx_trace::{TraceDecoder, TransactionTraceResponse};
use sncast::response::ui::UI;
use starknet_rust::core::types::{
    BlockId, BlockTag, ContractClass, ExecuteInvocation, FunctionInvocation, TransactionTrace,
};
use starknet_rust::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet_rust::providers::{Provider, ProviderError};
use starknet_types_core::felt::Felt;
use std::collections::{HashMap, HashSet};
use std::process::ExitCode;

const MAX_CONCURRENT_CLASS_REQUESTS: usize = 4;

struct FetchedContractClasses {
    classes: HashMap<Felt, ContractClass>,
    failures: Vec<ContractClassFetchFailure>,
}

struct ContractClassFetchFailure {
    class_hash: Felt,
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

impl TxTrace {
    fn validate(&self, output_format: OutputFormat) -> Result<()> {
        if self.full && output_format == OutputFormat::Json {
            bail!("`--full` cannot be used with `--json`");
        }

        Ok(())
    }
}

pub async fn tx_trace(tx_trace: TxTrace, config: CastConfig, ui: &UI) -> Result<ExitCode> {
    tx_trace.validate(ui.output_format())?;

    let provider = tx_trace.rpc.get_provider(&config, ui).await?;

    let result = provider
        .trace_transaction(tx_trace.transaction_hash)
        .await
        .map_err(|error| StarknetCommandError::ProviderError(error.into()))
        .map_err(handle_starknet_command_error);

    let result = match result {
        Ok(trace) => {
            let FetchedContractClasses { classes, failures } =
                fetch_contract_classes(&provider, class_hashes(&trace)).await;

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
    class_hashes: HashSet<Felt>,
) -> FetchedContractClasses {
    let results = stream::iter(class_hashes)
        .map(|class_hash| async move {
            match provider
                .get_class(BlockId::Tag(BlockTag::PreConfirmed), class_hash)
                .await
            {
                Ok(class) => Ok((class_hash, class)),
                Err(error) => Err(ContractClassFetchFailure { class_hash, error }),
            }
        })
        .buffer_unordered(MAX_CONCURRENT_CLASS_REQUESTS)
        .collect::<Vec<_>>()
        .await;

    let (classes, failures): (HashMap<_, _>, Vec<_>) = results.into_iter().partition_result();

    FetchedContractClasses { classes, failures }
}

fn class_hashes(transaction_trace: &TransactionTrace) -> HashSet<Felt> {
    let mut class_hashes = HashSet::new();
    for invocation in root_invocations(transaction_trace) {
        collect_class_hashes(invocation, &mut class_hashes);
    }
    class_hashes
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

fn collect_class_hashes(invocation: &FunctionInvocation, class_hashes: &mut HashSet<Felt>) {
    class_hashes.insert(invocation.class_hash);

    for nested_call in &invocation.calls {
        collect_class_hashes(nested_call, class_hashes);
    }
}

fn format_class_fetch_warning(failures: &[ContractClassFetchFailure]) -> String {
    let mut failures = failures.iter().collect::<Vec<_>>();
    failures.sort_unstable_by_key(|failure| failure.class_hash);

    let details = failures
        .into_iter()
        .map(|failure| {
            format!(
                "- class hash: {} — {}",
                failure.class_hash.to_hex_string(),
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
