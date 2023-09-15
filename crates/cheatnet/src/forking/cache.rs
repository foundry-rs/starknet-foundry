use bincode::{config, decode_from_slice, encode_to_vec, Decode, Encode};
use camino::Utf8PathBuf;
use fs2::FileExt;
use regex::Regex;
use starknet::core::types::BlockId;
use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use url::Url;

#[derive(Encode, Decode, PartialEq, Debug)]
pub struct ForkCache {
    pub cached_calls: HashMap<(String, Vec<String>), String>,
    pub cache_writes: HashMap<(String, Vec<String>), String>,
    pub cache_file: String,
}

impl ForkCache {
    #[must_use]
    pub fn load(url: &str, _block_id: BlockId) -> Self {
        let url = Url::parse(url).expect("Failed to parse URL");
        let url_str = url.as_str();
        let re = Regex::new(r"[^a-zA-Z0-9]").unwrap();

        // Use the replace_all method to replace non-alphanumeric characters with underscores
        let sanitized_path = re.replace_all(url_str, "_").to_string();

        // TODO: add block_id to the sanitized_path
        let path = Utf8PathBuf::from("./snfoundry_cache").join(sanitized_path + ".bin");

        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(&path)
            .unwrap();

        let mut contents = vec![];
        file.read_to_end(&mut contents).unwrap();

        let cached_calls: (HashMap<(String, Vec<String>), String>, usize) =
            decode_from_slice(&contents, config::standard()).unwrap_or((HashMap::new(), 0));

        ForkCache {
            cached_calls: cached_calls.0,
            cache_writes: HashMap::new(),
            cache_file: path.to_string(),
        }
    }

    pub fn save(&self) {
        let mut file = OpenOptions::new()
            .write(true)
            .open(&self.cache_file)
            .unwrap();
        file.lock_exclusive().unwrap();

        let contents = fs::read(&self.cache_file).expect("Should have been able to read the cache");
        let cached_calls: (HashMap<(String, Vec<String>), String>, usize) =
            decode_from_slice(&contents, config::standard()).unwrap_or((HashMap::new(), 0));
        let mut saved_calls = cached_calls.0;

        for (key, value) in &self.cache_writes {
            if !saved_calls.contains_key(key) {
                saved_calls.insert(key.clone(), value.clone());
            }
        }

        file.write_all(&encode_to_vec(saved_calls, config::standard()).unwrap())
            .unwrap();

        file.unlock().unwrap();
    }
}
