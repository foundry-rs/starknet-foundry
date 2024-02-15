use anyhow::Result;
use bimap::BiMap;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::get_class_hash;
use std::collections::HashMap;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use scarb_api::StarknetContractArtifacts;
use starknet_api::core::{ClassHash, EntryPointSelector};

#[derive(Debug, Clone)]
pub struct ContractsData {
    pub contracts: HashMap<String, StarknetContractArtifacts>,
    pub class_hashes: BiMap<String, ClassHash>,
    pub selectors: BiMap<String, EntryPointSelector>,
}

impl ContractsData {
    pub fn try_from(contracts: HashMap<String, StarknetContractArtifacts>) -> Result<Self> {
        let class_hashes: Vec<(String, ClassHash)> = contracts
            .par_iter()
            .map(|(name, artifact)| Ok((name.clone(), get_class_hash(artifact.sierra.as_str())?)))
            .collect::<Result<_>>()?;

        let selectors = BiMap::new();

        Ok(ContractsData {
            contracts,
            class_hashes: BiMap::from_iter(class_hashes),
            selectors,
        })
    }
}
