use anyhow::Result;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::get_class_hash;
use std::collections::HashMap;
use bimap::BiMap;

use scarb_api::StarknetContractArtifacts;
use starknet_api::core::{ClassHash, EntryPointSelector};






#[derive(Debug, Clone)]
pub struct ContractsData {
    pub contracts: HashMap<String, StarknetContractArtifacts>,
    pub class_hashes: BiMap<String, ClassHash>,
    pub selectors: BiMap<String, EntryPointSelector>
}

impl ContractsData {
    pub fn try_from(contracts: HashMap<String, StarknetContractArtifacts>) -> Result<Self> {
        let mut class_hashes: BiMap<String, ClassHash> = BiMap::new();
        for contract in &contracts {
            class_hashes.insert(contract.0.clone(), get_class_hash(contract.1.sierra.as_str())?);
        }

        let selectors = BiMap::new();

        Ok(ContractsData {
            contracts,
            class_hashes,
            selectors,
        })
    }
}

