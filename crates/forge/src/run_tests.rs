use crate::scarb::config::ForkTarget;
use anyhow::{anyhow, Result};
use forge_runner::compiled_runnable::{RawForkConfig, RawForkParams};
use with_config::with_config;

pub mod run_crate;
pub mod with_config;
pub mod workspace;

pub(crate) fn replace_id_with_params<'a>(
    raw_fork_config: &'a RawForkConfig,
    fork_targets: &'a [ForkTarget],
) -> Result<&'a RawForkParams> {
    match raw_fork_config {
        RawForkConfig::Params(raw_fork_params) => Ok(raw_fork_params),
        RawForkConfig::Id(name) => {
            let fork_target_from_runner_config = fork_targets
                .iter()
                .find(|fork| fork.name() == name)
                .ok_or_else(|| {
                    anyhow!("Fork configuration named = {name} not found in the Scarb.toml")
                })?;

            Ok(fork_target_from_runner_config.params())
        }
    }
}
