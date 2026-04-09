use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::Args;
use starknet_types_core::felt::Felt;

#[derive(Args, Debug, Clone)]
pub struct ProofArgs {
    /// Path to a file containing the proof (base64-encoded string) for the transaction.
    #[arg(long, requires = "proof_facts_file")]
    pub proof_file: Option<Utf8PathBuf>,

    /// Path to a file containing proof facts (comma-separated felts) for the transaction.
    #[arg(long, requires = "proof_file")]
    pub proof_facts_file: Option<Utf8PathBuf>,
}

impl ProofArgs {
    #[must_use]
    pub fn none() -> Self {
        Self {
            proof_file: None,
            proof_facts_file: None,
        }
    }

    pub fn resolve_proof(&self) -> Result<Option<String>> {
        match &self.proof_file {
            Some(path) => {
                let contents = std::fs::read_to_string(path)
                    .with_context(|| format!("Failed to read proof file {path}"))?;
                Ok(Some(strip_quotes(contents.trim()).to_string()))
            }
            None => Ok(None),
        }
    }

    pub fn resolve_proof_facts(&self) -> Result<Option<Vec<Felt>>> {
        match &self.proof_facts_file {
            Some(path) => {
                let contents = std::fs::read_to_string(path)
                    .with_context(|| format!("Failed to read proof facts file {path}"))?;
                let felts = contents
                    .split(',')
                    .map(|s| {
                        strip_quotes(s.trim())
                            .parse::<Felt>()
                            .with_context(|| format!("Failed to parse felt from '{}'", s.trim()))
                    })
                    .collect::<Result<Vec<_>>>()?;
                Ok(Some(felts))
            }
            None => Ok(None),
        }
    }
}

fn strip_quotes(value: &str) -> &str {
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        let first = bytes[0];
        let last = bytes[value.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return &value[1..value.len() - 1];
        }
    }
    value
}

impl Default for ProofArgs {
    fn default() -> Self {
        Self::none()
    }
}
