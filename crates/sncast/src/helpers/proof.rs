use clap::Args;
use starknet_types_core::felt::Felt;

#[derive(Args, Debug, Clone)]
pub struct ProofArgs {
    /// Base64-encoded proof for the transaction.
    #[arg(long)]
    pub proof: Option<String>,

    /// Proof facts for the transaction.
    #[arg(long, value_delimiter = ',', num_args = 1..)]
    pub proof_facts: Option<Vec<Felt>>,
}

impl ProofArgs {
    #[must_use]
    pub fn none() -> Self {
        Self {
            proof: None,
            proof_facts: None,
        }
    }
}

impl Default for ProofArgs {
    fn default() -> Self {
        Self::none()
    }
}
