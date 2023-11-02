use crate::forking::cache::ForkCache;
use crate::state::{BlockInfoReader, CheatnetBlockInfo};
use blockifier::execution::contract_class::{
    ContractClass as ContractClassBlockifier, ContractClassV0, ContractClassV1,
};
use blockifier::state::errors::StateError::{StateReadError, UndeclaredClassHash};
use blockifier::state::state_api::{StateReader, StateResult};
use cairo_lang_starknet::abi::Contract;
use cairo_lang_starknet::casm_contract_class::CasmContractClass;
use cairo_lang_starknet::contract_class::{ContractClass, ContractEntryPoints};
use cairo_lang_utils::bigint::BigUintAsHex;
use conversions::StarknetConversions;
use flate2::read::GzDecoder;
use num_bigint::BigUint;
use starknet::core::types::{
    BlockId, ContractClass as ContractClassStarknet, MaybePendingBlockWithTxHashes,
    PendingBlockWithTxHashes,
};
use starknet::providers::jsonrpc::{HttpTransport, JsonRpcClientError};
use starknet::providers::{JsonRpcClient, Provider, ProviderError};
use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::core::PatriciaKey;
use starknet_api::core::{ClassHash, CompiledClassHash, ContractAddress, Nonce};
use starknet_api::deprecated_contract_class::{
    ContractClass as DeprecatedContractClass, ContractClassAbiEntry, EntryPoint, EntryPointType,
};
use starknet_api::hash::StarkFelt;
use starknet_api::hash::StarkHash;
use starknet_api::patricia_key;
use starknet_api::state::StorageKey;
use std::collections::HashMap;
use std::io::Read;
use tokio::runtime::Runtime;
use url::Url;

#[derive(Debug)]
pub struct ForkStateReader {
    client: JsonRpcClient<HttpTransport>,
    block_id: BlockId,
    runtime: Runtime,
    cache: ForkCache,
}

impl ForkStateReader {
    #[must_use]
    pub fn new(url: Url, block_id: BlockId, cache_dir: Option<&str>) -> Self {
        ForkStateReader {
            cache: ForkCache::load_or_new(&url, block_id, cache_dir),
            client: JsonRpcClient::new(HttpTransport::new(url)),
            block_id,
            runtime: Runtime::new().expect("Could not instantiate Runtime"),
        }
    }
}

fn get_pending_block_parent(
    state_reader: &ForkStateReader,
    pending_block: &PendingBlockWithTxHashes,
) -> StateResult<CheatnetBlockInfo> {
    let parent_block_id = BlockId::Hash(pending_block.parent_hash);

    match state_reader.runtime.block_on(
        state_reader
            .client
            .get_block_with_tx_hashes(parent_block_id),
    ) {
        Ok(MaybePendingBlockWithTxHashes::Block(parent_block)) => Ok(CheatnetBlockInfo {
            block_number: BlockNumber(parent_block.block_number + 1),
            timestamp: BlockTimestamp(pending_block.timestamp),
            sequencer_address: ContractAddress(patricia_key!(pending_block.sequencer_address)),
        }),
        Ok(MaybePendingBlockWithTxHashes::PendingBlock(_)) => Err(StateReadError(
            "Parent block of the pending block cannot be pending".to_string(),
        )),
        Err(err) => Err(StateReadError(format!(
            "Unable to get parent block with tx hashes from fork, err: {err:?}"
        ))),
    }
}

impl BlockInfoReader for ForkStateReader {
    fn get_block_info(&mut self) -> StateResult<CheatnetBlockInfo> {
        if let Some(cache_hit) = self.cache.get_block_info() {
            return Ok(cache_hit);
        }

        match self
            .runtime
            .block_on(self.client.get_block_with_tx_hashes(self.block_id))
        {
            Ok(MaybePendingBlockWithTxHashes::Block(block)) => {
                let block_info = CheatnetBlockInfo {
                    block_number: BlockNumber(block.block_number),
                    timestamp: BlockTimestamp(block.timestamp),
                    sequencer_address: ContractAddress(patricia_key!(block.sequencer_address)),
                };

                self.cache.cache_get_block_info(block_info);

                Ok(block_info)
            }
            Ok(MaybePendingBlockWithTxHashes::PendingBlock(pending_block)) => {
                get_pending_block_parent(self, &pending_block)
            }
            Err(err) => Err(StateReadError(format!(
                "Unable to get block with tx hashes from fork, err: {err:?}"
            ))),
        }
    }
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
            contract_address.to_field_element(),
            key.0.key().to_field_element(),
            self.block_id,
        )) {
            Ok(value) => {
                let value_sf = value.to_stark_felt();
                self.cache
                    .cache_get_storage_at(contract_address, key, value_sf);
                Ok(value_sf)
            }
            Err(ProviderError::Other(JsonRpcClientError::TransportError(_))) => {
                node_connection_error()
            }
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
                .get_nonce(self.block_id, contract_address.to_field_element()),
        ) {
            Ok(nonce) => {
                let nonce = nonce.to_nonce();
                self.cache.cache_get_nonce_at(contract_address, nonce);
                Ok(nonce)
            }
            Err(ProviderError::Other(JsonRpcClientError::TransportError(_))) => {
                node_connection_error()
            }
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
                .get_class_hash_at(self.block_id, contract_address.to_field_element()),
        ) {
            Ok(class_hash) => {
                let class_hash = class_hash.to_class_hash();
                self.cache
                    .cache_get_class_hash_at(contract_address, class_hash);
                Ok(class_hash)
            }
            Err(ProviderError::Other(JsonRpcClientError::TransportError(_))) => {
                node_connection_error()
            }
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
                        .get_class(self.block_id, class_hash.to_field_element()),
                ) {
                    Ok(contract_class) => {
                        self.cache
                            .cache_get_compiled_contract_class(class_hash, &contract_class);

                        Ok(contract_class)
                    }
                    Err(ProviderError::Other(JsonRpcClientError::TransportError(_))) => {
                        node_connection_error()
                    }
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
                let converted_entry_points: ContractEntryPoints = serde_json::from_str(
                    &serde_json::to_string(&flattened_class.entry_points_by_type).unwrap(),
                )
                .unwrap();
                let converted_abi: Contract = serde_json::from_str(&flattened_class.abi).unwrap();

                let sierra_contract_class: ContractClass = ContractClass {
                    sierra_program: converted_sierra_program,
                    sierra_program_debug_info: None,
                    contract_class_version: flattened_class.contract_class_version,
                    entry_points_by_type: converted_entry_points,
                    abi: Some(converted_abi),
                };
                let casm_contract_class: CasmContractClass =
                    CasmContractClass::from_contract_class(sierra_contract_class, false).unwrap();

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
                let converted_abi: Option<Vec<ContractClassAbiEntry>> =
                    serde_json::from_str(&serde_json::to_string(&legacy_class.abi).unwrap())
                        .unwrap();

                let mut decoder = GzDecoder::new(&legacy_class.program[..]);
                let mut converted_program = String::new();
                decoder.read_to_string(&mut converted_program).unwrap();

                Ok(ContractClassBlockifier::V0(
                    ContractClassV0::try_from(DeprecatedContractClass {
                        abi: converted_abi,
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

fn node_connection_error<T>() -> StateResult<T> {
    Err(StateReadError(
        "Unable to reach the node. Check your internet connection and node url".to_string(),
    ))
}
