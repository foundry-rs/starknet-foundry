use anyhow::Result;
use clap::Args;
use starknet_rust::core::types::FeeEstimate;

use crate::response::dry_run::DryRunResponse;

#[derive(Args, Debug, Clone)]
pub struct DryRunArgs {
    /// If passed, the transaction will not be sent to the network and the fee will be estimated instead.
    #[arg(long)]
    pub dry_run: bool,

    /// If passed, the output will include detailed fee estimation results instead of just overall fee. Only works with `--dry-run` flag.
    #[arg(long, requires = "dry_run")]
    pub detailed: bool,
}

impl DryRunArgs {
    pub async fn estimate_if_dry_run<T, E, Fut>(
        &self,
        estimate_fee: impl FnOnce() -> Fut,
        into_response: impl FnOnce(DryRunResponse) -> T,
    ) -> Option<Result<T, E>>
    where
        Fut: std::future::Future<Output = Result<FeeEstimate, E>>,
    {
        if !self.dry_run {
            return None;
        }
        Some(
            estimate_fee()
                .await
                .map(|e| into_response(DryRunResponse::new(&e, self.detailed))),
        )
    }
}
