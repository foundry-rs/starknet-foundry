use cairo_vm::Felt252;
use conversions::{byte_array::ByteArray, serde::deserialize::CairoDeserialize};
use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer,
};
use std::str::FromStr;
use std::{fmt, num::NonZeroU32};
use url::Url;
// available gas

#[derive(Debug, Clone, CairoDeserialize)]
pub struct RawAvailableGasConfig {
    pub gas: usize,
}

// fork

#[derive(Debug, Clone, CairoDeserialize, PartialEq)]
pub enum BlockId {
    BlockTag,
    BlockHash(Felt252),
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
                            Felt252::from_str(&map.next_value::<String>()?)
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
                            ))
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
    ShortString(Felt252),
    ByteArray(ByteArray),
    Array(Vec<Felt252>),
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

// config

#[derive(Debug, Default, Clone)]
pub struct RawForgeConfig {
    pub fork: Option<RawForkConfig>,
    pub available_gas: Option<RawAvailableGasConfig>,
    pub ignore: Option<RawIgnoreConfig>,
    pub should_panic: Option<RawShouldPanicConfig>,
    pub fuzzer: Option<RawFuzzerConfig>,
}
