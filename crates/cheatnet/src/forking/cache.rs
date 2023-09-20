use camino::Utf8PathBuf;
use conversions::StarknetConversions;
use regex::Regex;
use serde::{Deserialize, Serialize};
use starknet::core::types::{BlockId, BlockTag, ContractClass, FieldElement};
use starknet_api::core::{ClassHash, ContractAddress, Nonce};
use starknet_api::hash::StarkFelt;
use starknet_api::state::StorageKey;
use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
struct ForkCacheContent {
    cache_version: String,
    storage_at: HashMap<String, HashMap<String, String>>,
    nonce_at: HashMap<String, String>,
    class_hash_at: HashMap<String, String>,
    compiled_contract_class: HashMap<String, String>,
    compiled_class_hash: HashMap<String, String>,
}

impl ForkCacheContent {
    fn new() -> Self {
        Self {
            cache_version: "1.0".to_string(),
            storage_at: HashMap::new(),
            nonce_at: HashMap::new(),
            class_hash_at: HashMap::new(),
            compiled_contract_class: HashMap::new(),
            compiled_class_hash: HashMap::new(),
        }
    }
    fn from_str(serialized: &str) -> Self {
        serde_json::from_str(serialized).expect("Could not deserialize json cache")
    }

    fn to_string(&self) -> String {
        serde_json::to_string(self).expect("Could not serialize json cache")
    }

    fn apply(&mut self, other: &Self) {
        for (other_contract_address, other_storage) in other.storage_at.iter() {
            if let Some(self_contract_storage) = self.storage_at.get(other_contract_address) {
                let mut new_storage = self_contract_storage.clone();
                new_storage.extend(other_storage.clone());
                self.storage_at
                    .insert(other_contract_address.clone(), new_storage);
            } else {
                self.storage_at
                    .insert(other_contract_address.clone(), other_storage.clone());
            }
        }
    }
}

#[derive(Debug)]
pub struct ForkCache {
    fork_cache_content: ForkCacheContent,
    cache_file: String,
}

fn block_id_to_string(block_id: BlockId) -> String {
    match block_id {
        BlockId::Hash(x) => x.to_felt252().to_str_radix(16),
        BlockId::Number(x) => x.to_string(),
        BlockId::Tag(x) => match x {
            BlockTag::Latest => "latest".to_string(),
            BlockTag::Pending => "pending".to_string(),
        },
    }
}

impl ForkCache {
    #[must_use]
    pub(crate) fn load(url: &str, block_id: BlockId) -> Self {
        let url = Url::parse(url).expect("Failed to parse URL");
        let url_str = url.as_str();
        let re = Regex::new(r"[^a-zA-Z0-9]").unwrap();

        // Use the replace_all method to replace non-alphanumeric characters with underscores
        let sanitized_path = re.replace_all(url_str, "_").to_string();

        let path = Utf8PathBuf::from("./.snforge_cache")
            .join(sanitized_path + "_" + block_id_to_string(block_id).as_str() + ".json");

        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(&path)
            .unwrap();

        let mut cache_file_content: String = "".to_string();
        file.read_to_string(&mut cache_file_content)
            .expect("Could not read cache file: {path}");

        let fork_cache_content = if cache_file_content == "" {
            ForkCacheContent::new()
        } else {
            ForkCacheContent::from_str(cache_file_content.as_str())
        };

        ForkCache {
            fork_cache_content,
            cache_file: path.to_string(),
        }
    }

    pub fn save(&self) {
        let mut file = OpenOptions::new()
            .write(true)
            .open(&self.cache_file)
            .unwrap();

        file.lock_exclusive().expect("Could not lock on cache file");

        let cache_file_content =
            fs::read_to_string(&self.cache_file).expect("Should have been able to read the cache");

        let output = if cache_file_content != "".to_string() {
            let mut fs_fork_cache_content = ForkCacheContent::from_str(cache_file_content.as_str());
            fs_fork_cache_content.apply(&self.fork_cache_content);
            fs_fork_cache_content.to_string()
        } else {
            self.fork_cache_content.to_string()
        };

        file.write_all(output.as_bytes())
            .expect("Could not write cache to file");

        file.unlock().unwrap();
    }

    pub(crate) fn get_storage_at(
        &self,
        contract_address: ContractAddress,
        key: StorageKey,
    ) -> Option<StarkFelt> {
        let contract_address_str = contract_address.to_felt252().to_string();
        let storage_key_str = key.0.key().to_felt252().to_string();

        let cache_hit = self
            .fork_cache_content
            .storage_at
            .get(&contract_address_str)?
            .get(&storage_key_str)?;

        Some(
            FieldElement::from_hex_be(cache_hit)
                .expect(format!("Could not parse {cache_hit}").as_str())
                .to_stark_felt(),
        )
    }

    pub(crate) fn cache_get_storage_at(
        &mut self,
        contract_address: ContractAddress,
        key: StorageKey,
        value: StarkFelt,
    ) {
        let contract_address_str = contract_address.to_felt252().to_string();
        let storage_key_str = key.0.key().to_felt252().to_string();
        let value_str = value.to_felt252().to_string();

        if !self
            .fork_cache_content
            .storage_at
            .contains_key(&contract_address_str)
        {
            self.fork_cache_content
                .storage_at
                .insert(contract_address_str.clone(), HashMap::new());
        };

        self.fork_cache_content
            .storage_at
            .get_mut(&contract_address_str)
            .unwrap()
            .insert(storage_key_str.clone(), value_str.clone());
    }

    pub(crate) fn get_nonce_at(&self, address: ContractAddress) -> Option<Nonce> {
        self.fork_cache_content
            .nonce_at
            .get(&address.to_felt252().to_string())
            .map(|cache_hit| cache_hit.to_nonce())
    }

    pub(crate) fn cache_get_nonce_at(&mut self, contract_address: ContractAddress, nonce: Nonce) {
        let contract_address_str = contract_address.to_felt252().to_string();
        let nonce_str = nonce.to_felt252().to_string();

        self.fork_cache_content
            .nonce_at
            .insert(contract_address_str, nonce_str);
    }

    pub(crate) fn get_class_hash_at(&self, contract_address: ContractAddress) -> Option<ClassHash> {
        self.fork_cache_content
            .nonce_at
            .get(&contract_address.to_felt252().to_string())
            .map(|cache_hit| cache_hit.to_class_hash())
    }

    pub(crate) fn cache_get_class_hash_at(
        &mut self,
        contract_address: ContractAddress,
        class_hash: ClassHash,
    ) {
        let contract_address_str = contract_address.to_felt252().to_string();
        let class_hash_str = class_hash.to_felt252().to_string();

        self.fork_cache_content
            .class_hash_at
            .insert(contract_address_str, class_hash_str);
    }

    pub(crate) fn get_compiled_contract_class(
        &self,
        class_hash: &ClassHash,
    ) -> Option<ContractClass> {
        let class_hash = class_hash.to_felt252().to_string();
        self.fork_cache_content
            .compiled_contract_class
            .get(&class_hash)
            .map(|cache_hit| {
                serde_json::from_str(cache_hit).expect("Could not parse the ContractClass")
            })
    }

    pub(crate) fn cache_get_compiled_contract_class(
        &mut self,
        class_hash: &ClassHash,
        contract_class: ContractClass,
    ) {
        let class_hash_str = class_hash.to_felt252().to_string();
        let contract_class_str = serde_json::to_string(&contract_class)
            .expect("Could not serialize ContractClassV1 into string");
        self.fork_cache_content
            .compiled_contract_class
            .insert(class_hash_str, contract_class_str);
    }
}
