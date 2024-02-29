use crate::compiled_raw::RawForkParams;
use anyhow::{bail, Result};
use itertools::Itertools;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::struct_excuessive_bools)]
#[derive(Debug, PartialEq, Default)]
pub struct ForgeConfig {
    /// Should runner exit after first failed test
    pub exit_first: bool,
    /// How many runs should fuzzer execute
    pub fuzzer_runs: Option<u32>,
    /// Seed to be used by fuzzer
    pub fuzzer_seed: Option<u64>,
    /// Display more detailed info about used resources
    pub detailed_resources: bool,
    /// Save execution traces of all test which have passed and are not fuzz tests
    pub save_trace_data: bool,
    /// Builds profile of all test which have passed and are not fuzz tests
    pub build_profile: bool,
    /// Fork configuration profiles
    pub fork: Vec<ForkTarget>,
    /// Limit of steps
    pub max_n_steps: Option<u32>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ForkTarget {
    name: String,
    params: RawForkParams,
}

impl ForkTarget {
    #[must_use]
    pub fn new(name: String, params: RawForkParams) -> Self {
        Self { name, params }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn params(&self) -> &RawForkParams {
        &self.params
    }
}

/// Represents forge config deserialized from Scarb.toml using basic types like String etc.
#[allow(clippy::struct_excessive_bools)]
#[derive(Deserialize, Debug, PartialEq, Default)]
pub(crate) struct RawForgeConfig {
    #[serde(default)]
    /// Should runner exit after first failed test
    pub exit_first: bool,
    /// How many runs should fuzzer execute
    pub fuzzer_runs: Option<u32>,
    /// Seed to be used by fuzzer
    pub fuzzer_seed: Option<u64>,
    #[serde(default)]
    // Display more detailed info about used resources
    pub detailed_resources: bool,
    #[serde(default)]
    /// Save execution traces of all test which have passed and are not fuzz tests
    pub save_trace_data: bool,
    #[serde(default)]
    /// Builds profiles of all test which have passed and are not fuzz tests
    pub build_profile: bool,
    #[serde(default)]
    /// Fork configuration profiles
    pub fork: Vec<RawForkTarget>,
    /// Limit of steps
    pub max_n_steps: Option<u32>,
}

#[derive(Deserialize, Debug, PartialEq, Default, Clone)]
pub(crate) struct RawForkTarget {
    pub name: String,
    pub url: String,
    pub block_id: HashMap<String, String>,
}

fn validate_raw_fork_config(raw_config: RawForgeConfig) -> Result<RawForgeConfig> {
    let forks = &raw_config.fork;

    let names: Vec<_> = forks.iter().map(|fork| &fork.name).collect();
    let removed_duplicated_names: HashSet<_> = names.iter().collect();

    if names.len() != removed_duplicated_names.len() {
        bail!("Some fork names are duplicated");
    }

    for fork in forks {
        let block_id_item = fork.block_id.iter().exactly_one();

        let Ok((block_id_key, block_id_value)) = block_id_item else {
            bail!("block_id should be set once per fork");
        };

        if !["number", "hash", "tag"].contains(&&**block_id_key) {
            bail!("block_id = {block_id_key} is not valid. Possible values are = \"number\", \"hash\" and \"tag\"");
        }

        if block_id_key == "tag" && block_id_value != "Latest" {
            bail!("block_id.tag can only be equal to Latest");
        }
    }

    Ok(raw_config)
}

impl TryFrom<RawForgeConfig> for ForgeConfig {
    type Error = anyhow::Error;

    fn try_from(value: RawForgeConfig) -> Result<Self, Self::Error> {
        let value = validate_raw_fork_config(value)?;
        let mut fork_targets = vec![];

        for raw_fork_target in value.fork {
            let (block_id_type, block_id_value) =
                raw_fork_target.block_id.iter().exactly_one().unwrap();

            fork_targets.push(ForkTarget::new(
                raw_fork_target.name,
                RawForkParams {
                    url: raw_fork_target.url,
                    block_id_type: block_id_type.to_string(),
                    block_id_value: block_id_value.clone(),
                },
            ));
        }

        Ok(ForgeConfig {
            exit_first: value.exit_first,
            fuzzer_runs: value.fuzzer_runs,
            fuzzer_seed: value.fuzzer_seed,
            detailed_resources: value.detailed_resources,
            save_trace_data: value.save_trace_data,
            build_profile: value.build_profile,
            fork: fork_targets,
            max_n_steps: value.max_n_steps,
        })
    }
}
