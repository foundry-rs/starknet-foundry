use crate::TestArgs;
use anyhow::{Result, ensure};
use clap::Parser;
use std::num::NonZeroU32;

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
    pub step: NonZeroU32,

    /// Maximum allowed contract file size in bytes
    #[arg(long, default_value = "4089446")]
    pub max_contract_size: u64,

    /// Maximum allowed length of compiled contract program.
    #[arg(long, default_value = "81920")]
    pub max_contract_program_len: u64,

    /// Update Scarb.toml with the threshold that minimizes runtime gas cost
    #[arg(long, conflicts_with = "size")]
    pub gas: bool,

    /// Update Scarb.toml with the threshold that minimizes contract size cost
    #[arg(long, conflicts_with = "gas")]
    pub size: bool,

    /// Comma-delimited list of contract names or Cairo paths (e.g. `MyContract,pkg::MyOther`)
    /// to include in contract size checks.
    #[arg(long, value_delimiter = ',', required = true)]
    pub contracts: Vec<String>,

    /// Test arguments (same as for `snforge test`)
    #[command(flatten)]
    pub test_args: TestArgs,
}

impl OptimizeInliningArgs {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.test_args.exact,
            "optimize-inlining requires using the `--exact` flag"
        );
        ensure!(
            self.min_threshold <= self.max_threshold,
            "min-threshold ({}) must be <= max-threshold ({})",
            self.min_threshold,
            self.max_threshold
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::OptimizeInliningArgs;
    use clap::Parser;

    #[test]
    fn validation_fails_without_exact() {
        let args =
            OptimizeInliningArgs::parse_from(["snforge", "--contracts", "MyContract", "test_name"]);
        let error = args.validate().unwrap_err().to_string();
        assert!(error.contains("optimize-inlining requires using the `--exact` flag"));
    }

    #[test]
    fn validation_fails_without_test_name() {
        let args = OptimizeInliningArgs::try_parse_from([
            "snforge",
            "--exact",
            "--contracts",
            "MyContract",
        ]);
        let error = args.unwrap_err().to_string();
        assert!(error.contains(
            "error: the following required arguments were not provided:\n  <TEST_FILTER>"
        ));
    }

    #[test]
    fn validation_passes_with_single_exact_test_name() {
        let args = OptimizeInliningArgs::parse_from([
            "snforge",
            "--exact",
            "--contracts",
            "MyContract",
            "test_name",
        ]);
        args.validate().unwrap();
    }
}
