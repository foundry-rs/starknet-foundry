use crate::TestArgs;
use anyhow::{Result, ensure};
use clap::Parser;

#[derive(Parser, Debug)]
pub struct OptimizeInliningArgs {
    /// Minimum inlining-strategy value to test
    #[arg(long, default_value = "0")]
    pub min_threshold: u32,

    /// Maximum inlining-strategy value to test
    #[arg(long, default_value = "250")]
    pub max_threshold: u32,

    /// Step size for threshold search
    #[arg(long, default_value = "25")]
    pub step: u32,

    /// Maximum allowed contract size in bytes
    #[arg(long, default_value = "4089446")]
    pub max_contract_size: u64,

    /// Maximum allowed contract felts count
    #[arg(long, default_value = "81920")]
    pub max_contract_felts: u64,

    /// Update Scarb.toml with the threshold that minimizes runtime gas cost
    #[arg(long, conflicts_with = "size")]
    pub gas: bool,

    /// Update Scarb.toml with the threshold that minimizes contract size cost
    #[arg(long, conflicts_with = "gas")]
    pub size: bool,

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
        self.test_args.validate_single_exact_test_case()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::OptimizeInliningArgs;
    use clap::Parser;

    #[test]
    fn validation_fails_without_exact() {
        let args = OptimizeInliningArgs::parse_from(["snforge", "test_name"]);
        let error = args.validate().unwrap_err().to_string();
        assert!(error.contains("requires --exact"));
    }

    #[test]
    fn validation_fails_without_test_name() {
        let args = OptimizeInliningArgs::try_parse_from(["snforge", "--exact"]);
        let error = args.unwrap_err().to_string();
        assert!(error.contains(
            "error: the following required arguments were not provided:\n  <TEST_FILTER>"
        ));
    }

    #[test]
    fn validation_passes_with_single_exact_test_name() {
        let args = OptimizeInliningArgs::parse_from(["snforge", "--exact", "test_name"]);
        args.validate().unwrap();
    }
}
