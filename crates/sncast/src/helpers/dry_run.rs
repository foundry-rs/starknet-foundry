use clap::Args;
use starknet_rust::core::types::FeeEstimate;
use std::future::Future;

use crate::response::dry_run::DryRunResponse;

#[derive(Args, Debug, Clone, Default)]
pub struct DryRunArgs {
    /// If passed, the transaction will not be sent to the network and the fee will be estimated instead.
    #[arg(long, conflicts_with_all = ["max_fee", "l1_gas", "l1_gas_price", "l2_gas", "l2_gas_price", "l1_data_gas", "l1_data_gas_price", "tip", "estimate_tip"])]
    pub dry_run: bool,

    /// If passed, the output will include detailed fee estimation results instead of just overall fee. Only works with `--dry-run` flag.
    #[arg(long, requires = "dry_run")]
    pub detailed: bool,
}

impl DryRunArgs {
    pub async fn estimate<E, Fut>(
        &self,
        estimate_fee: impl FnOnce() -> Fut,
    ) -> Result<DryRunResponse, E>
    where
        Fut: Future<Output = Result<FeeEstimate, E>>,
    {
        estimate_fee()
            .await
            .map(|fee| DryRunResponse::new(&fee, self.detailed))
    }
}
