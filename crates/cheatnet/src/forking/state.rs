use crate::forking::cache::ForkCache;
use crate::state::BlockInfoReader;
use blockifier::execution::contract_class::{
    ContractClass as ContractClassBlockifier, ContractClassV0, ContractClassV1,
};
use blockifier::state::errors::StateError::{StateReadError, UndeclaredClassHash};
use blockifier::state::state_api::{StateReader, StateResult};
use cairo_lang_utils::bigint::BigUintAsHex;
use conversions::{FromConv, IntoConv};
use flate2::read::GzDecoder;
use num_bigint::BigUint;
use runtime::starknet::context::BlockInfo;
use sierra_casm::compile;
use starknet::core::types::{
    BlockId, ContractClass as ContractClassStarknet, FieldElement, MaybePendingBlockWithTxHashes,
};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider, ProviderError};
use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::core::{ClassHash, CompiledClassHash, ContractAddress, Nonce};
use starknet_api::deprecated_contract_class::{
    ContractClass as DeprecatedContractClass, EntryPoint, EntryPointType,
};
use starknet_api::hash::StarkFelt;
use starknet_api::state::StorageKey;
use std::collections::HashMap;
use std::io::Read;
use std::ops::Deref;
use tokio::runtime::Runtime;
use url::Url;

#[derive(Debug)]
pub struct ForkStateReader {
    client: JsonRpcClient<HttpTransport>,
    block_number: BlockNumber,
    runtime: Runtime,
    cache: ForkCache,
}

impl ForkStateReader {
    #[must_use]
    pub fn new(url: Url, block_number: BlockNumber, cache_dir: &str) -> Self {
        ForkStateReader {
            cache: ForkCache::load_or_new(&url, block_number, cache_dir),
            client: JsonRpcClient::new(HttpTransport::new(url)),
            block_number,
            runtime: Runtime::new().expect("Could not instantiate Runtime"),
        }
    }

    fn block_id(&self) -> BlockId {
        BlockId::Number(self.block_number.0)
    }
}

impl BlockInfoReader for ForkStateReader {
    fn get_block_info(&mut self) -> StateResult<BlockInfo> {
        if let Some(cache_hit) = self.cache.get_block_info() {
            return Ok(cache_hit);
        }

        match self
            .runtime
            .block_on(self.client.get_block_with_tx_hashes(self.block_id()))
        {
            Ok(MaybePendingBlockWithTxHashes::Block(block)) => {
                let block_info = BlockInfo {
                    block_number: BlockNumber(block.block_number),
                    timestamp: BlockTimestamp(block.timestamp),
                    sequencer_address: block.sequencer_address.into_(),
                };

                self.cache.cache_get_block_info(block_info);

                Ok(block_info)
            }
            Ok(MaybePendingBlockWithTxHashes::PendingBlock(_)) => {
                unreachable!("Pending block is not be allowed at the configuration level")
            }
            Err(err) => Err(StateReadError(format!(
                "Unable to get block with tx hashes from fork, err: {err:?}"
            ))),
        }
    }
}

#[macro_export]
macro_rules! other_provider_error {
    ( $boxed:expr ) => {{
        let err_str = $boxed.deref().to_string();
        if err_str.contains("error sending request for url") {
            return node_connection_error();
        }
        Err(StateReadError(format!("JsonRpc provider error: {err_str}")))
    }};
}

fn node_connection_error<T>() -> StateResult<T> {
    Err(StateReadError(
        "Unable to reach the node. Check your internet connection and node url".to_string(),
    ))
}

impl StateReader for ForkStateReader {
    fn get_storage_at(
        &mut self,
        contract_address: ContractAddress,
        key: StorageKey,
    ) -> StateResult<StarkFelt> {
        if let Some(cache_hit) = self.cache.get_storage_at(contract_address, key) {
            return Ok(cache_hit);
        }

        match self.runtime.block_on(self.client.get_storage_at(
            FieldElement::from_(contract_address),
            FieldElement::from_(*key.0.key()),
            self.block_id(),
        )) {
            Ok(value) => {
                let value_sf: StarkFelt = value.into_();
                self.cache
                    .cache_get_storage_at(contract_address, key, value_sf);
                Ok(value_sf)
            }
            Err(ProviderError::Other(boxed)) => other_provider_error!(boxed),
            Err(_) => Err(StateReadError(format!(
                "Unable to get storage at address: {contract_address:?} and key: {key:?} from fork"
            ))),
        }
    }

    fn get_nonce_at(&mut self, contract_address: ContractAddress) -> StateResult<Nonce> {
        if let Some(cache_hit) = self.cache.get_nonce_at(contract_address) {
            return Ok(cache_hit);
        }

        match self.runtime.block_on(
            self.client
                .get_nonce(self.block_id(), FieldElement::from_(contract_address)),
        ) {
            Ok(nonce) => {
                let nonce = nonce.into_();
                self.cache.cache_get_nonce_at(contract_address, nonce);
                Ok(nonce)
            }
            Err(ProviderError::Other(boxed)) => other_provider_error!(boxed),
            Err(_) => Err(StateReadError(format!(
                "Unable to get nonce at {contract_address:?} from fork"
            ))),
        }
    }

    fn get_class_hash_at(&mut self, contract_address: ContractAddress) -> StateResult<ClassHash> {
        if let Some(cache_hit) = self.cache.get_class_hash_at(contract_address) {
            return Ok(cache_hit);
        }

        match self.runtime.block_on(
            self.client
                .get_class_hash_at(self.block_id(), FieldElement::from_(contract_address)),
        ) {
            Ok(class_hash) => {
                let class_hash: ClassHash = class_hash.into_();
                self.cache
                    .cache_get_class_hash_at(contract_address, class_hash);
                Ok(class_hash)
            }
            Err(ProviderError::Other(boxed)) => other_provider_error!(boxed),
            Err(_) => Err(StateReadError(format!(
                "Unable to get class hash at {contract_address:?} from fork"
            ))),
        }
    }

    fn get_compiled_contract_class(
        &mut self,
        class_hash: &ClassHash,
    ) -> StateResult<ContractClassBlockifier> {
        let contract_class =
            if let Some(cache_hit) = self.cache.get_compiled_contract_class(class_hash) {
                Ok(cache_hit)
            } else {
                match self.runtime.block_on(
                    self.client
                        .get_class(self.block_id(), FieldElement::from_(*class_hash)),
                ) {
                    Ok(contract_class) => {
                        self.cache
                            .cache_get_compiled_contract_class(class_hash, &contract_class);

                        Ok(contract_class)
                    }
                    Err(ProviderError::Other(boxed)) => other_provider_error!(boxed),
                    Err(_) => Err(UndeclaredClassHash(*class_hash)),
                }
            };

        match contract_class? {
            ContractClassStarknet::Sierra(flattened_class) => {
                let converted_sierra_program: Vec<BigUintAsHex> = flattened_class
                    .sierra_program
                    .iter()
                    .map(|field_element| BigUintAsHex {
                        value: BigUint::from_bytes_be(&field_element.to_bytes_be()),
                    })
                    .collect();

                let sierra_contract_class = serde_json::json!({
                    "sierra_program": converted_sierra_program,
                    "contract_class_version": "",
                    "entry_points_by_type": flattened_class.entry_points_by_type
                });

                let casm_contract_class = compile(sierra_contract_class).unwrap();

                Ok(ContractClassBlockifier::V1(
                    ContractClassV1::try_from(casm_contract_class).unwrap(),
                ))
            }
            ContractClassStarknet::Legacy(legacy_class) => {
                let converted_entry_points: HashMap<EntryPointType, Vec<EntryPoint>> =
                    serde_json::from_str(
                        &serde_json::to_string(&legacy_class.entry_points_by_type).unwrap(),
                    )
                    .unwrap();

                let mut decoder = GzDecoder::new(&legacy_class.program[..]);
                let mut converted_program = String::new();
                decoder.read_to_string(&mut converted_program).unwrap();

                Ok(ContractClassBlockifier::V0(
                    ContractClassV0::try_from(DeprecatedContractClass {
                        abi: None,
                        program: serde_json::from_str(&converted_program).unwrap(),
                        entry_points_by_type: converted_entry_points,
                    })
                    .unwrap(),
                ))
            }
        }
    }

    fn get_compiled_class_hash(
        &mut self,
        _class_hash: ClassHash,
    ) -> StateResult<CompiledClassHash> {
        Err(StateReadError(
            "Unable to get compiled class hash from the fork".to_string(),
        ))
    }
}
