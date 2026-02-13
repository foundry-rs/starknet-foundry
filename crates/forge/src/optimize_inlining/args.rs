use crate::TestArgs;
use anyhow::{Result, ensure};
use clap::Parser;

#[derive(Parser, Debug)]
pub struct OptimizeInliningArgs {
    /// Minimum inlining-strategy value to test
    #[arg(long, default_value = "0")]
    pub min_threshold: u32,

    /// Maximum inlining-strategy value to test
    #[arg(long, default_value = "100")]
    pub max_threshold: u32,

    /// Step size for threshold search
    #[arg(long, default_value = "10")]
    pub step: u32,

    /// Maximum allowed contract size in bytes
    #[arg(long, default_value = "4089446")]
    pub max_contract_size: u64,

    /// Maximum allowed contract felts count
    #[arg(long, default_value = "81920")]
    pub max_contract_felts: u64,

    /// Dry run - don't modify Scarb.toml at the end
    #[arg(long, default_value = "false")]
    pub dry_run: bool,

    /// Test arguments (same as `snforge test`)
    #[command(flatten)]
    pub test_args: TestArgs,
}

impl OptimizeInliningArgs {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.min_threshold <= self.max_threshold,
            "min-threshold ({}) must be <= max-threshold ({})",
            self.min_threshold,
            self.max_threshold
        );
        ensure!(self.step > 0, "step must be greater than 0");
        Ok(())
    }
}
