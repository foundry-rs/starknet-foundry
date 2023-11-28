use cairo_felt::Felt252;
use starknet_api::block::BlockNumber;
use std::collections::HashMap;

#[derive(Default)]
pub struct BlockNumberMap {
    url_to_latest_block_number: HashMap<String, BlockNumber>,
    url_and_hash_to_block_number: HashMap<(String, Felt252), BlockNumber>,
}

impl BlockNumberMap {
    pub fn add_latest_block_number(&mut self, url: String, latest_block_number: BlockNumber) {
        self.url_to_latest_block_number
            .insert(url, latest_block_number);
    }

    pub fn add_block_number_for_hash(
        &mut self,
        url: String,
        hash: Felt252,
        block_number: BlockNumber,
    ) {
        self.url_and_hash_to_block_number
            .insert((url, hash), block_number);
    }

    pub fn get_latest_block_number(&mut self, url: &String) -> Option<&BlockNumber> {
        self.url_to_latest_block_number.get(url)
    }

    pub fn get_block_number_for_hash(
        &mut self,
        url: String,
        hash: Felt252,
    ) -> Option<&BlockNumber> {
        self.url_and_hash_to_block_number.get(&(url, hash))
    }

    #[must_use]
    pub fn get_url_to_latest_block_number(&self) -> &HashMap<String, BlockNumber> {
        &self.url_to_latest_block_number
    }
}
