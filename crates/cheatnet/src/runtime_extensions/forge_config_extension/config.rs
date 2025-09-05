use conversions::{byte_array::ByteArray, serde::deserialize::CairoDeserialize};
use serde::{
    Deserialize, Deserializer,
    de::{self, MapAccess, Visitor},
};
use starknet_api::execution_resources::{GasAmount, GasVector};
use starknet_types_core::felt::Felt;
use std::str::FromStr;
use std::{fmt, num::NonZeroU32};
use url::Url;
// available gas

#[derive(Debug, Clone, Copy, CairoDeserialize, PartialEq)]
pub struct RawAvailableResourceBoundsConfig {
    pub l1_gas: usize,
    pub l1_data_gas: usize,
    pub l2_gas: usize,
}

impl RawAvailableResourceBoundsConfig {
    #[must_use]
    pub fn to_gas_vector(&self) -> GasVector {
        GasVector {
            l1_gas: GasAmount(self.l1_gas as u64),
            l1_data_gas: GasAmount(self.l1_data_gas as u64),
            l2_gas: GasAmount(self.l2_gas as u64),
        }
    }

    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.to_gas_vector() == GasVector::ZERO
    }
}

// fork

#[derive(Debug, Clone, CairoDeserialize, PartialEq)]
pub enum BlockId {
    BlockTag,
    BlockHash(Felt),
    BlockNumber(u64),
}

impl<'de> Deserialize<'de> for BlockId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BlockIdVisitor;

        impl<'de> Visitor<'de> for BlockIdVisitor {
            type Value = BlockId;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map with exactly one of: tag, hash, or number")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut block_id = None;

                while let Some(key) = map.next_key::<String>()? {
                    if block_id.is_some() {
                        return Err(de::Error::custom(
                            "block_id must contain exactly one key: 'tag', 'hash', or 'number'",
                        ));
                    }

                    block_id = Some(match key.as_str() {
                        "tag" => {
                            let tag = map.next_value::<String>()?;
                            if tag != "latest" {
                                return Err(de::Error::custom(
                                    "block_id.tag can only be equal to latest",
                                ));
                            }
                            BlockId::BlockTag
                        }
                        "hash" => BlockId::BlockHash(
                            Felt::from_str(&map.next_value::<String>()?)
                                .map_err(de::Error::custom)?,
                        ),
                        "number" => BlockId::BlockNumber(
                            map.next_value::<String>()?
                                .parse()
                                .map_err(de::Error::custom)?,
                        ),
                        unknown => {
                            return Err(de::Error::unknown_field(
                                unknown,
                                &["tag", "hash", "number"],
                            ));
                        }
                    });
                }

                block_id.ok_or_else(|| de::Error::missing_field("block_id"))
            }
        }

        deserializer.deserialize_map(BlockIdVisitor)
    }
}

#[derive(Debug, Clone, CairoDeserialize, PartialEq)]
pub struct InlineForkConfig {
    pub url: Url,
    pub block: BlockId,
}

#[derive(Debug, Clone, CairoDeserialize, PartialEq)]
pub struct OverriddenForkConfig {
    pub name: ByteArray,
    pub block: BlockId,
}

#[derive(Debug, Clone, CairoDeserialize, PartialEq)]
pub enum RawForkConfig {
    Inline(InlineForkConfig),
    Named(ByteArray),
    Overridden(OverriddenForkConfig),
}

// fuzzer

#[derive(Debug, Clone, CairoDeserialize, PartialEq)]
pub struct RawFuzzerConfig {
    pub runs: Option<NonZeroU32>,
    pub seed: Option<u64>,
}

// should panic

#[derive(Debug, Clone, CairoDeserialize)]
pub enum Expected {
    ShortString(Felt),
    ByteArray(ByteArray),
    Array(Vec<Felt>),
    Any,
}

#[derive(Debug, Clone, CairoDeserialize)]
pub struct RawShouldPanicConfig {
    pub expected: Expected,
}

// ignore

#[derive(Debug, Clone, CairoDeserialize)]
pub struct RawIgnoreConfig {
    pub is_ignored: bool,
}

// disable strk predeployment

#[derive(Debug, Clone, CairoDeserialize)]
pub struct RawPredeployedContractsConfig {
    pub is_disabled: bool,
}

// config

#[derive(Debug, Default, Clone)]
pub struct RawForgeConfig {
    pub fork: Option<RawForkConfig>,
    pub available_gas: Option<RawAvailableResourceBoundsConfig>,
    pub ignore: Option<RawIgnoreConfig>,
    pub should_panic: Option<RawShouldPanicConfig>,
    pub fuzzer: Option<RawFuzzerConfig>,
    pub disable_predeployed_contracts: Option<RawPredeployedContractsConfig>,
}
