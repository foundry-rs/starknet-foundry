/// Contains compiled Starknet artifacts
#[derive(Debug, Clone)]
pub struct CastStarknetContractArtifacts {
    /// Compiled sierra code
    pub sierra: String,
    /// Compiled casm code
    pub casm: String,
}
