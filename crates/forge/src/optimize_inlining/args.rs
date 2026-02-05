use anyhow::{Result, ensure};
use crate::TestArgs;
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

    /// Weight for gas in the scoring formula (percent, 0-100). Felts weight is automatically set to (100 - gas_weight). Cannot be used with --felts-weight.
    #[arg(long, conflicts_with = "felts_weight")]
    pub gas_weight: Option<u8>,

    /// Weight for felts in the scoring formula (percent, 0-100). Gas weight is automatically set to (100 - felts_weight). Cannot be used with --gas-weight.
    #[arg(long, conflicts_with = "gas_weight")]
    pub felts_weight: Option<u8>,

    /// Use brute force search instead of Brent optimization
    #[arg(long, default_value = "false")]
    pub bruteforce: bool,

    /// Dry run - don't modify Scarb.toml at the end
    #[arg(long, default_value = "false")]
    pub dry_run: bool,

    /// Test arguments (same as `snforge test`)
    #[command(flatten)]
    pub test_args: TestArgs,
}

impl OptimizeInliningArgs {
    pub fn validate(&self) -> Result<()> {
        if let Some(gas) = self.gas_weight {
            ensure!(
                gas <= 100,
                "gas-weight ({}) must be between 0 and 100",
                gas
            );
        }
        if let Some(felts) = self.felts_weight {
            ensure!(
                felts <= 100,
                "felts-weight ({}) must be between 0 and 100",
                felts
            );
        }
        Ok(())
    }

    pub fn gas_weight(&self) -> u8 {
        match (self.gas_weight, self.felts_weight) {
            (Some(gas), None) => gas,
            (None, Some(felts)) => 100 - felts,
            (None, None) => 75, // default
            (Some(_), Some(_)) => unreachable!("clap conflicts_with prevents this"),
        }
    }

    pub fn felts_weight(&self) -> u8 {
        100 - self.gas_weight()
    }
}
