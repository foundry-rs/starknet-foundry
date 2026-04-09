use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::Args;
use starknet_types_core::felt::Felt;

#[derive(Args, Debug, Clone, Default)]
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
                    .with_context(|| format!("Failed to read proof file at {path}"))?;
                Ok(Some(strip_quotes(contents.trim()).to_string()))
            }
            None => Ok(None),
        }
    }

    pub fn resolve_proof_facts(&self) -> Result<Option<Vec<Felt>>> {
        match &self.proof_facts_file {
            Some(path) => {
                let contents = std::fs::read_to_string(path)
                    .with_context(|| format!("Failed to read proof facts file at {path}"))?;
                let felts = contents
                    .split(',')
                    .map(|s| {
                        let stripped = strip_quotes(s.trim());
                        stripped
                            .parse::<Felt>()
                            .with_context(|| format!("Failed to parse felt from '{stripped}'"))
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


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn proof_file() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "SGVsbG8gd29ybGQhCg==").unwrap();
        let path = Utf8PathBuf::try_from(file.path().to_path_buf()).unwrap();

        let args = ProofArgs {
            proof_file: Some(path),
            ..Default::default()
        };
        let result = args.resolve_proof().unwrap();

        assert_eq!(result, Some("SGVsbG8gd29ybGQhCg==".to_string()));
    }

    #[test]
    fn missing_proof_file() {
        let missing_path = "/nonexistent/proof.txt";
        let args = ProofArgs {
            proof_file: Some(Utf8PathBuf::from(missing_path)),
            ..Default::default()
        };
        let err = args.resolve_proof().unwrap_err().to_string();

        assert!(err.contains(&format!("Failed to read proof file at {missing_path}")));
    }

    #[test]
    fn proof_facts_file() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "0x1, 0x2, 0x3").unwrap();
        let path = Utf8PathBuf::try_from(file.path().to_path_buf()).unwrap();

        let args = ProofArgs {
            proof_facts_file: Some(path),
            ..Default::default()
        };
        let result = args.resolve_proof_facts().unwrap();
        assert_eq!(
            result,
            Some(vec![Felt::from(1), Felt::from(2), Felt::from(3)])
        );
    }

    #[test]
    fn proof_facts_file_quoted() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "\"0x1\", '0x2', 0x3").unwrap();
        let path = Utf8PathBuf::try_from(file.path().to_path_buf()).unwrap();

        let args = ProofArgs {
            proof_facts_file: Some(path),
            ..Default::default()
        };
        let result = args.resolve_proof_facts().unwrap();

        assert_eq!(
            result,
            Some(vec![Felt::from(1), Felt::from(2), Felt::from(3)])
        );
    }

    #[test]
    fn proof_facts_malformed() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "0x1, invalid, 0x3").unwrap();
        let path = Utf8PathBuf::try_from(file.path().to_path_buf()).unwrap();

        let args = ProofArgs {
            proof_facts_file: Some(path),
            ..Default::default()
        };
        let err = args.resolve_proof_facts().unwrap_err().to_string();

        assert!(err.contains("Failed to parse felt from 'invalid'"));
    }

    #[test]
    fn missing_proof_facts() {
        let missing_path = "/nonexistent/path/proof_facts.txt";
        let args = ProofArgs {
            proof_facts_file: Some(Utf8PathBuf::from(missing_path)),
            ..Default::default()
        };
        let err = args.resolve_proof_facts().unwrap_err().to_string();

        assert!(err.contains(&format!(
            "Failed to read proof facts file at {missing_path}"
        )));
    }
}
