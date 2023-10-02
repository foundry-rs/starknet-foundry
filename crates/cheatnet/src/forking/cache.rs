use camino::Utf8PathBuf;
use conversions::StarknetConversions;
use fs2::FileExt;
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
        serde_json::from_str(serialized).expect("Could not deserialize cache from json")
    }

    fn extend(&mut self, other: &Self) {
        // storage_at
        for (other_contract_address, other_storage) in &other.storage_at {
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
        // nonce_at
        self.nonce_at.extend(other.nonce_at.clone());
        // class_hash_at
        self.class_hash_at.extend(other.class_hash_at.clone());
        // compiled_contract_class
        self.compiled_contract_class
            .extend(other.compiled_contract_class.clone());
        // compiled_class_hash
        self.compiled_class_hash
            .extend(other.compiled_class_hash.clone());
    }
}

impl ToString for ForkCacheContent {
    fn to_string(&self) -> String {
        serde_json::to_string(self).expect("Could not serialize json cache")
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ForkCache {
    fork_cache_content: ForkCacheContent,
    cache_file: Option<String>,
    block_id: BlockId,
}

impl Drop for ForkCache {
    fn drop(&mut self) {
        if !matches!(self.block_id, BlockId::Tag(_)) {
            self.save();
        }
    }
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
    pub(crate) fn load_or_new(url: &str, block_id: BlockId, cache_dir: Option<&str>) -> Self {
        let (fork_cache_content, cache_file) = if let BlockId::Tag(_) = block_id {
            (ForkCacheContent::new(), None)
        } else {
            let cache_file_path = cache_file_path_from_fork_config(url, block_id, cache_dir);
            let mut file = OpenOptions::new()
                .write(true)
                .read(true)
                .create(true)
                .open(&cache_file_path)
                .unwrap();

            let mut cache_file_content: String = String::new();
            file.read_to_string(&mut cache_file_content)
                .expect("Could not read cache file: {path}");

            // File was just created
            let fork_cache_content = if cache_file_content.is_empty() {
                ForkCacheContent::new()
            } else {
                ForkCacheContent::from_str(cache_file_content.as_str())
            };

            (fork_cache_content, Some(cache_file_path.to_string()))
        };

        ForkCache {
            fork_cache_content,
            cache_file,
            block_id,
        }
    }

    fn save(&self) {
        let cache_file = self
            .cache_file
            .clone()
            .unwrap_or_else(|| panic!("No cache_file to save to"));
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(cache_file.clone())
            .unwrap();

        file.lock_exclusive().expect("Could not lock on cache file");

        let cache_file_content =
            fs::read_to_string(cache_file).expect("Should have been able to read the cache");

        let output = if cache_file_content.is_empty() {
            self.fork_cache_content.to_string()
        } else {
            let mut fs_fork_cache_content = ForkCacheContent::from_str(cache_file_content.as_str());
            fs_fork_cache_content.extend(&self.fork_cache_content);
            fs_fork_cache_content.to_string()
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
                .unwrap_or_else(|_| panic!("Could not parse {cache_hit}"))
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

        self.fork_cache_content
            .storage_at
            .entry(contract_address_str.clone())
            .or_insert_with(HashMap::new);

        self.fork_cache_content
            .storage_at
            .get_mut(&contract_address_str)
            .unwrap()
            .insert(storage_key_str, value_str);
    }

    pub(crate) fn get_nonce_at(&self, address: ContractAddress) -> Option<Nonce> {
        self.fork_cache_content
            .nonce_at
            .get(&address.to_felt252().to_string())
            .map(StarknetConversions::to_nonce)
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
            .map(StarknetConversions::to_class_hash)
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
        contract_class: &ContractClass,
    ) {
        let class_hash_str = class_hash.to_felt252().to_string();
        let contract_class_str = serde_json::to_string(&contract_class)
            .expect("Could not serialize ContractClassV1 into string");
        self.fork_cache_content
            .compiled_contract_class
            .insert(class_hash_str, contract_class_str);
    }
}

fn cache_file_path_from_fork_config(
    url: &str,
    block_id: BlockId,
    cache_dir: Option<&str>,
) -> Utf8PathBuf {
    let cache_dir = cache_dir.unwrap_or_else(|| {
        panic!("cache_dir has to be provided if working at a concrete block_number/block_hash")
    });
    let url = Url::parse(url).expect("Failed to parse URL");
    let url_str = url.as_str();
    let re = Regex::new(r"[^a-zA-Z0-9]").unwrap();

    // Use the replace_all method to replace non-alphanumeric characters with underscores
    let sanitized_path = re.replace_all(url_str, "_").to_string();

    let cache_file_path = Utf8PathBuf::from(cache_dir)
        .join(sanitized_path + "_" + block_id_to_string(block_id).as_str() + ".json");

    fs::create_dir_all(cache_file_path.parent().unwrap())
        .expect("Fork cache directory could not be created");

    cache_file_path
}
