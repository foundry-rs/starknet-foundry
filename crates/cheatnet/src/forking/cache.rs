use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use fs2::FileExt;
use regex::Regex;
use runtime::starknet::context::SerializableBlockInfo;
use serde::{Deserialize, Serialize};
use starknet::core::types::ContractClass;
use starknet_api::block::{BlockInfo, BlockNumber};
use starknet_api::core::{ClassHash, ContractAddress, Nonce};
use starknet_api::state::StorageKey;
use starknet_types_core::felt::Felt;
use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::string::ToString;
use std::sync::{Arc, Mutex};
use url::Url;

#[must_use]
pub fn cache_version() -> String {
    env!("CARGO_PKG_VERSION").replace('.', "_")
}

#[derive(Serialize, Deserialize, Debug)]
struct ForkCacheContent {
    cache_version: String,
    storage_at: HashMap<ContractAddress, HashMap<StorageKey, Felt>>,
    nonce_at: HashMap<ContractAddress, Nonce>,
    class_hash_at: HashMap<ContractAddress, ClassHash>,
    compiled_contract_class: HashMap<ClassHash, ContractClass>,
    block_info: Option<SerializableBlockInfo>,
}

impl Default for ForkCacheContent {
    fn default() -> Self {
        Self {
            cache_version: cache_version(),
            storage_at: HashMap::default(),
            nonce_at: HashMap::default(),
            class_hash_at: HashMap::default(),
            compiled_contract_class: HashMap::default(),
            block_info: Option::default(),
        }
    }
}

impl ForkCacheContent {
    fn from_str(serialized: &str) -> Self {
        let cache: Self =
            serde_json::from_str(serialized).expect("Could not deserialize cache from json");

        assert_eq!(
            cache.cache_version,
            cache_version(),
            "Expected the Version {}",
            cache_version()
        );

        cache
    }

    fn extend(&mut self, other: &Self) {
        // storage_at
        for (other_contract_address, other_storage) in &other.storage_at {
            if let Some(self_contract_storage) = self.storage_at.get_mut(other_contract_address) {
                self_contract_storage.extend(other_storage.clone());
            } else {
                self.storage_at
                    .insert(*other_contract_address, other_storage.clone());
            }
        }

        self.nonce_at.extend(other.nonce_at.clone());
        self.class_hash_at.extend(other.class_hash_at.clone());
        self.compiled_contract_class
            .extend(other.compiled_contract_class.clone());
        if other.block_info.is_some() {
            self.block_info.clone_from(&other.block_info);
        }
    }
}

impl Display for ForkCacheContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self).expect("Could not serialize json cache")
        )
    }
}

#[derive(Debug)]
pub struct CacheDir {
    dir: Mutex<Utf8PathBuf>,
}

#[cfg(feature = "testing")]
impl PartialEq for CacheDir {
    fn eq(&self, other: &Self) -> bool {
        *self.dir.lock().unwrap() == *other.dir.lock().unwrap()
    }
}

impl CacheDir {
    #[must_use]
    pub fn new(path: Utf8PathBuf) -> Self {
        Self {
            dir: Mutex::new(path),
        }
    }

    fn load(&self, url: &Url, block_number: BlockNumber) -> Result<ForkCacheContent> {
        let dir = self.dir.lock().unwrap();

        let cache_file = cache_file_path_from_fork_config(url, block_number, &dir)?;
        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .truncate(false)
            .open(&cache_file)
            .context("Could not open cache file")?;

        let mut cache_file_content = String::new();
        file.read_to_string(&mut cache_file_content)
            .context("Could not read cache file")?;

        // File was just created
        if cache_file_content.is_empty() {
            Ok(ForkCacheContent::default())
        } else {
            Ok(ForkCacheContent::from_str(cache_file_content.as_str()))
        }
    }

    fn save(
        &self,
        url: &Url,
        block_number: BlockNumber,
        fork_cache_content: &ForkCacheContent,
    ) -> Result<()> {
        let dir = self.dir.lock().unwrap();

        let cache_file = cache_file_path_from_fork_config(url, block_number, &dir)?;

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(&cache_file)?;

        file.lock_exclusive().expect("Could not lock on cache file");

        let cache_file_content =
            fs::read_to_string(&cache_file).expect("Should have been able to read the cache");

        let output = if cache_file_content.is_empty() {
            fork_cache_content.to_string()
        } else {
            let mut fs_fork_cache_content = ForkCacheContent::from_str(&cache_file_content);
            fs_fork_cache_content.extend(fork_cache_content);
            fs_fork_cache_content.to_string()
        };

        file.write_all(output.as_bytes())
            .expect("Could not write cache to file");

        fs2::FileExt::unlock(&file)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct ForkCache {
    fork_cache_content: ForkCacheContent,
    cache_dir: Arc<CacheDir>,
    block_number: BlockNumber,
    url: Url,
}

impl Drop for ForkCache {
    fn drop(&mut self) {
        self.save();
    }
}

impl ForkCache {
    pub(crate) fn load_or_new(
        url: &Url,
        block_number: BlockNumber,
        cache_dir: Arc<CacheDir>,
    ) -> Result<Self> {
        let fork_cache_content = cache_dir.load(url, block_number)?;
        Ok(Self {
            fork_cache_content,
            cache_dir,
            block_number,
            url: url.clone(),
        })
    }

    fn save(&mut self) {
        self.cache_dir
            .save(&self.url, self.block_number, &self.fork_cache_content)
            .expect("Failed to save");
    }

    pub(crate) fn get_storage_at(
        &self,
        contract_address: &ContractAddress,
        key: &StorageKey,
    ) -> Option<Felt> {
        self.fork_cache_content
            .storage_at
            .get(contract_address)?
            .get(key)
            .copied()
    }

    pub(crate) fn cache_get_storage_at(
        &mut self,
        contract_address: ContractAddress,
        key: StorageKey,
        value: Felt,
    ) {
        self.fork_cache_content
            .storage_at
            .entry(contract_address)
            .or_default()
            .insert(key, value);
    }

    pub(crate) fn get_nonce_at(&self, address: &ContractAddress) -> Option<Nonce> {
        self.fork_cache_content.nonce_at.get(address).copied()
    }

    pub(crate) fn cache_get_nonce_at(&mut self, contract_address: ContractAddress, nonce: Nonce) {
        self.fork_cache_content
            .nonce_at
            .insert(contract_address, nonce);
    }

    pub(crate) fn get_class_hash_at(
        &self,
        contract_address: &ContractAddress,
    ) -> Option<ClassHash> {
        self.fork_cache_content
            .class_hash_at
            .get(contract_address)
            .copied()
    }

    pub(crate) fn cache_get_class_hash_at(
        &mut self,
        contract_address: ContractAddress,
        class_hash: ClassHash,
    ) {
        self.fork_cache_content
            .class_hash_at
            .insert(contract_address, class_hash);
    }

    pub(crate) fn get_compiled_contract_class(
        &self,
        class_hash: &ClassHash,
    ) -> Option<&ContractClass> {
        self.fork_cache_content
            .compiled_contract_class
            .get(class_hash)
    }

    pub(crate) fn insert_compiled_contract_class(
        &mut self,
        class_hash: ClassHash,
        contract_class: ContractClass,
    ) -> &ContractClass {
        self.fork_cache_content
            .compiled_contract_class
            .entry(class_hash)
            .or_insert(contract_class)
    }

    pub(crate) fn get_block_info(&self) -> Option<BlockInfo> {
        Some(self.fork_cache_content.block_info.clone()?.into())
    }

    pub(crate) fn cache_get_block_info(&mut self, block_info: BlockInfo) {
        self.fork_cache_content.block_info = Some(block_info.into());
    }
}

fn cache_file_path_from_fork_config(
    url: &Url,
    BlockNumber(block_number): BlockNumber,
    cache_dir: &Utf8Path,
) -> Result<Utf8PathBuf> {
    let re = Regex::new(r"[^a-zA-Z0-9]").unwrap();

    // replace non-alphanumeric characters with underscores
    let sanitized_path = re.replace_all(url.as_str(), "_");

    let cache_file_path = cache_dir.join(format!(
        "{sanitized_path}_{block_number}_v{}.json",
        cache_version()
    ));

    fs::create_dir_all(cache_file_path.parent().unwrap())
        .context("Fork cache directory could not be created")?;

    Ok(cache_file_path)
}
