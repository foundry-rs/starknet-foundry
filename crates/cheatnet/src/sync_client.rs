use starknet::core::types::{BlockId, ContractClass, MaybePendingBlockWithTxHashes};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider, ProviderError};
use starknet_api::block::BlockNumber;
use starknet_types_core::felt::Felt;
use tokio::runtime::Runtime;
use url::Url;

#[derive(Debug)]
pub struct SyncClient {
    client: JsonRpcClient<HttpTransport>,
    block_id: BlockId,
    runtime: Runtime,
}

impl SyncClient {
    #[must_use]
    pub fn new(url: Url, block_number: BlockNumber) -> Self {
        Self {
            client: JsonRpcClient::new(HttpTransport::new(url)),
            block_id: BlockId::Number(block_number.0),
            runtime: Runtime::new().expect("Could not instantiate Runtime"),
        }
    }

    pub fn chain_id(&self) -> Result<Felt, ProviderError> {
        self.sync(self.client.chain_id())
    }

    pub fn get_block_with_tx_hashes(&self) -> Result<MaybePendingBlockWithTxHashes, ProviderError> {
        self.sync(self.client.get_block_with_tx_hashes(self.block_id))
    }

    pub fn get_storage_at(&self, contract_address: Felt, key: Felt) -> Result<Felt, ProviderError> {
        self.sync(
            self.client
                .get_storage_at(contract_address, key, self.block_id),
        )
    }

    pub fn get_nonce(&self, contract_address: Felt) -> Result<Felt, ProviderError> {
        self.sync(self.client.get_nonce(self.block_id, contract_address))
    }

    pub fn get_class_hash_at(&self, contract_address: Felt) -> Result<Felt, ProviderError> {
        self.sync(
            self.client
                .get_class_hash_at(self.block_id, contract_address),
        )
    }

    pub fn get_class(&self, class_hash: Felt) -> Result<ContractClass, ProviderError> {
        self.sync(self.client.get_class(self.block_id, class_hash))
    }

    fn sync<F: Future>(&self, future: F) -> F::Output {
        self.runtime.block_on(future)
    }
}
