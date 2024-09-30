use cairo_vm::Felt252;
use conversions::{byte_array::ByteArray, serde::deserialize::CairoDeserialize};
use std::num::NonZeroU32;
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
