use crate::forking::cache::ForkCache;
use crate::state::BlockInfoReader;
use anyhow::{Context, Result};
use blockifier::execution::contract_class::{
    CompiledClassV0, CompiledClassV0Inner, CompiledClassV1, RunnableCompiledClass,
};
use blockifier::state::errors::StateError::{self, StateReadError, UndeclaredClassHash};
use blockifier::state::state_api::{StateReader, StateResult};
use cairo_lang_utils::bigint::BigUintAsHex;
use cairo_vm::types::program::Program;
use camino::Utf8Path;
use conversions::{FromConv, IntoConv};
use flate2::read::GzDecoder;
use num_bigint::BigUint;
use runtime::starknet::context::SerializableGasPrices;
use starknet::core::types::{
    BlockId, ContractClass as ContractClassStarknet, MaybePendingBlockWithTxHashes, StarknetError,
};
use starknet::core::utils::parse_cairo_short_string;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider, ProviderError};
use starknet_api::block::{BlockInfo, BlockNumber, BlockTimestamp};
use starknet_api::contract_class::SierraVersion;
use starknet_api::core::{ChainId, ClassHash, CompiledClassHash, ContractAddress, Nonce};
use starknet_api::state::StorageKey;
use starknet_types_core::felt::Felt;
use std::cell::RefCell;
use std::io::Read;
use std::sync::Arc;
use tokio::runtime::Runtime;
use universal_sierra_compiler_api::{SierraType, compile_sierra};
use url::Url;

#[derive(Debug)]
pub struct ForkStateReader {
    client: JsonRpcClient<HttpTransport>,
    block_number: BlockNumber,
    runtime: Runtime,
    cache: RefCell<ForkCache>,
}

impl ForkStateReader {
    pub fn new(url: Url, block_number: BlockNumber, cache_dir: &Utf8Path) -> Result<Self> {
        Ok(ForkStateReader {
            cache: RefCell::new(
                ForkCache::load_or_new(&url, block_number, cache_dir)
                    .context("Could not create fork cache")?,
            ),
            client: JsonRpcClient::new(HttpTransport::new(url)),
            block_number,
            runtime: Runtime::new().expect("Could not instantiate Runtime"),
        })
    }

    pub fn chain_id(&self) -> Result<ChainId> {
        let id = self.runtime.block_on(self.client.chain_id())?;
        let id = parse_cairo_short_string(&id)?;
        Ok(ChainId::from(id))
    }

    fn block_id(&self) -> BlockId {
        BlockId::Number(self.block_number.0)
    }
}

#[expect(clippy::needless_pass_by_value)]
fn other_provider_error<T>(boxed: impl ToString) -> Result<T, StateError> {
    let err_str = boxed.to_string();

    Err(StateReadError(
        if err_str.contains("error sending request for url") {
            "Unable to reach the node. Check your internet connection and node url".to_string()
        } else {
            format!("JsonRpc provider error: {err_str}")
        },
    ))
}

impl BlockInfoReader for ForkStateReader {
    fn get_block_info(&mut self) -> StateResult<BlockInfo> {
        if let Some(cache_hit) = self.cache.borrow().get_block_info() {
            return Ok(cache_hit);
        }

        match self
            .runtime
            .block_on(self.client.get_block_with_tx_hashes(self.block_id()))
        {
            Ok(MaybePendingBlockWithTxHashes::Block(block)) => {
                let block_info = BlockInfo {
                    block_number: BlockNumber(block.block_number),
                    sequencer_address: block.sequencer_address.into_(),
                    block_timestamp: BlockTimestamp(block.timestamp),
                    gas_prices: SerializableGasPrices::default().into(),
                    use_kzg_da: true,
                };

                self.cache
                    .borrow_mut()
                    .cache_get_block_info(block_info.clone());

                Ok(block_info)
            }
            Ok(MaybePendingBlockWithTxHashes::PendingBlock(_)) => {
                unreachable!("Pending block is not be allowed at the configuration level")
            }
            Err(ProviderError::Other(boxed)) => other_provider_error(boxed),
            Err(err) => Err(StateReadError(format!(
                "Unable to get block with tx hashes from fork ({err})"
            ))),
        }
    }
}

impl StateReader for ForkStateReader {
    fn get_storage_at(
        &self,
        contract_address: ContractAddress,
        key: StorageKey,
    ) -> StateResult<Felt> {
        if let Some(cache_hit) = self.cache.borrow().get_storage_at(&contract_address, &key) {
            return Ok(cache_hit);
        }

        match self.runtime.block_on(self.client.get_storage_at(
            Felt::from_(contract_address),
            Felt::from_(*key.0.key()),
            self.block_id(),
        )) {
            Ok(value) => {
                let value_sf = value.into_();
                self.cache
                    .borrow_mut()
                    .cache_get_storage_at(contract_address, key, value_sf);
                Ok(value_sf)
            }
            Err(ProviderError::Other(boxed)) => other_provider_error(boxed),
            Err(ProviderError::StarknetError(StarknetError::ContractNotFound)) => {
                self.cache.borrow_mut().cache_get_storage_at(
                    contract_address,
                    key,
                    Felt::default(),
                );
                Ok(Felt::default())
            }
            Err(x) => Err(StateReadError(format!(
                "Unable to get storage at address: {contract_address:?} and key: {key:?} from fork ({x})"
            ))),
        }
    }

    fn get_nonce_at(&self, contract_address: ContractAddress) -> StateResult<Nonce> {
        if let Some(cache_hit) = self.cache.borrow().get_nonce_at(&contract_address) {
            return Ok(cache_hit);
        }

        match self.runtime.block_on(
            self.client
                .get_nonce(self.block_id(), Felt::from_(contract_address)),
        ) {
            Ok(nonce) => {
                let nonce = nonce.into_();
                self.cache
                    .borrow_mut()
                    .cache_get_nonce_at(contract_address, nonce);
                Ok(nonce)
            }
            Err(ProviderError::Other(boxed)) => other_provider_error(boxed),
            Err(ProviderError::StarknetError(StarknetError::ContractNotFound)) => {
                self.cache
                    .borrow_mut()
                    .cache_get_nonce_at(contract_address, Nonce::default());
                Ok(Nonce::default())
            }
            Err(x) => Err(StateReadError(format!(
                "Unable to get nonce at {contract_address:?} from fork ({x})"
            ))),
        }
    }

    fn get_class_hash_at(&self, contract_address: ContractAddress) -> StateResult<ClassHash> {
        if let Some(cache_hit) = self.cache.borrow().get_class_hash_at(&contract_address) {
            return Ok(cache_hit);
        }

        match self.runtime.block_on(
            self.client
                .get_class_hash_at(self.block_id(), Felt::from_(contract_address)),
        ) {
            Ok(class_hash) => {
                let class_hash = class_hash.into_();
                self.cache
                    .borrow_mut()
                    .cache_get_class_hash_at(contract_address, class_hash);
                Ok(class_hash)
            }
            Err(ProviderError::StarknetError(StarknetError::ContractNotFound)) => {
                self.cache
                    .borrow_mut()
                    .cache_get_class_hash_at(contract_address, ClassHash::default());
                Ok(ClassHash::default())
            }
            Err(ProviderError::Other(boxed)) => other_provider_error(boxed),
            Err(x) => Err(StateReadError(format!(
                "Unable to get class hash at {contract_address:?} from fork ({x})"
            ))),
        }
    }

    fn get_compiled_class(&self, class_hash: ClassHash) -> StateResult<RunnableCompiledClass> {
        let mut cache = self.cache.borrow_mut();

        let contract_class = {
            if let Some(cache_hit) = cache.get_compiled_contract_class(&class_hash) {
                Ok(cache_hit)
            } else {
                match self.runtime.block_on(
                    self.client
                        .get_class(self.block_id(), Felt::from_(class_hash)),
                ) {
                    Ok(contract_class) => {
                        Ok(cache.insert_compiled_contract_class(class_hash, contract_class))
                    }
                    Err(ProviderError::StarknetError(StarknetError::ClassHashNotFound)) => {
                        Err(UndeclaredClassHash(class_hash))
                    }
                    Err(ProviderError::Other(boxed)) => other_provider_error(boxed),
                    Err(x) => Err(StateReadError(format!(
                        "Unable to get compiled class at {class_hash} from fork ({x})"
                    ))),
                }
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

                match compile_sierra::<String>(&sierra_contract_class, &SierraType::Contract) {
                    Ok(casm_contract_class_raw) => Ok(RunnableCompiledClass::V1(
                        CompiledClassV1::try_from_json_string(
                            &casm_contract_class_raw,
                            SierraVersion::LATEST,
                        )
                        .expect("Unable to create RunnableCompiledClass::V1"),
                    )),
                    Err(err) => Err(StateReadError(err.to_string())),
                }
            }
            ContractClassStarknet::Legacy(legacy_class) => {
                let converted_entry_points = serde_json::from_str(
                    &serde_json::to_string(&legacy_class.entry_points_by_type).unwrap(),
                )
                .unwrap();

                let mut decoder = GzDecoder::new(&legacy_class.program[..]);
                let mut converted_program = String::new();
                decoder.read_to_string(&mut converted_program).unwrap();

                Ok(RunnableCompiledClass::V0(CompiledClassV0(Arc::new(
                    CompiledClassV0Inner {
                        program: Program::from_bytes(converted_program.as_ref(), None)
                            .expect("Unable to load program from converted_program"),
                        entry_points_by_type: converted_entry_points,
                    },
                ))))
            }
        }
    }

    fn get_compiled_class_hash(&self, _class_hash: ClassHash) -> StateResult<CompiledClassHash> {
        Err(StateReadError(
            "Unable to get compiled class hash from the fork".to_string(),
        ))
    }
}
