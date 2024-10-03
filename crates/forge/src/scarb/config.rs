use anyhow::{anyhow, bail, Result};
use cheatnet::runtime_extensions::forge_config_extension::config::BlockId;
use itertools::Itertools;
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    num::NonZeroU32,
};
use url::Url;

pub const SCARB_MANIFEST_TEMPLATE_CONTENT: &str = r#"
# Visit https://foundry-rs.github.io/starknet-foundry/appendix/scarb-toml.html for more information

# [tool.snforge]                                             # Define `snforge` tool section
# exit_first = true                                          # Stop tests execution immediately upon the first failure
# fuzzer_runs = 1234                                         # Number of runs of the random fuzzer
# fuzzer_seed = 1111                                         # Seed for the random fuzzer

# [[tool.snforge.fork]]                                      # Used for fork testing
# name = "SOME_NAME"                                         # Fork name
# url = "http://your.rpc.url"                                # Url of the RPC provider
# block_id.tag = "latest"                                    # Block to fork from (block tag)

# [[tool.snforge.fork]]
# name = "SOME_SECOND_NAME"
# url = "http://your.second.rpc.url"                         
# block_id.number = "123"                                    # Block to fork from (block number)

# [[tool.snforge.fork]]
# name = "SOME_THIRD_NAME"
# url = "http://your.third.rpc.url"
# block_id.hash = "0x123"                                    # Block to fork from (block hash)

# [profile.dev.cairo]                                        # Configure Cairo compiler
# unstable-add-statements-code-locations-debug-info = true   # Should be used if you want to use coverage
# unstable-add-statements-functions-debug-info = true        # Should be used if you want to use coverage/profiler
# inlining-strategy = "avoid"                                # Should be used if you want to use coverage

# [features]                                                 # Used for conditional compilation
# enable_for_tests = []                                      # Feature name and list of other features that should be enabled with it
"#;

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, PartialEq, Default)]
pub struct ForgeConfigFromScarb {
    /// Should runner exit after first failed test
    pub exit_first: bool,
    /// How many runs should fuzzer execute
    pub fuzzer_runs: Option<NonZeroU32>,
    /// Seed to be used by fuzzer
    pub fuzzer_seed: Option<u64>,
    /// Display more detailed info about used resources
    pub detailed_resources: bool,
    /// Save execution traces of all test which have passed and are not fuzz tests
    pub save_trace_data: bool,
    /// Build profiles of all tests which have passed and are not fuzz tests
    pub build_profile: bool,
    /// Generate a coverage report for the executed tests which have passed and are not fuzz tests
    pub coverage: bool,
    /// Fork configuration profiles
    pub fork: Vec<ForkTarget>,
    /// Limit of steps
    pub max_n_steps: Option<u32>,
}

#[non_exhaustive]
#[derive(Debug, PartialEq, Clone)]
pub struct ForkTarget {
    pub name: String,
    pub url: Url,
    pub block_id: BlockId,
}

impl ForkTarget {
    pub fn new(name: &str, url: &str, block_id_type: &str, block_id_value: &str) -> Result<Self> {
        let parsed_url = Url::parse(url).map_err(|_| anyhow!("Failed to parse fork url"))?;
        let block_id = match block_id_type {
            "number" => BlockId::BlockNumber(
                block_id_value
                    .parse()
                    .map_err(|_| anyhow!("Failed to parse block number"))?,
            ),
            "hash" => BlockId::BlockHash(
                block_id_value
                    .parse()
                    .map_err(|_| anyhow!("Failed to parse block hash"))?,
            ),
            "tag" => match block_id_value {
                "latest" => BlockId::BlockTag,
                _ => bail!("block_id.tag can only be equal to latest"),
            },
            block_id_key => bail!("block_id = {block_id_key} is not valid. Possible values are = \"number\", \"hash\" and \"tag\""),
        };

        Ok(Self {
            name: name.to_string(),
            url: parsed_url,
            block_id,
        })
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
    pub fuzzer_runs: Option<NonZeroU32>,
    /// Seed to be used by fuzzer
    pub fuzzer_seed: Option<u64>,
    #[serde(default)]
    // Display more detailed info about used resources
    pub detailed_resources: bool,
    #[serde(default)]
    /// Save execution traces of all test which have passed and are not fuzz tests
    pub save_trace_data: bool,
    #[serde(default)]
    /// Build profiles of all tests which have passed and are not fuzz tests
    pub build_profile: bool,
    #[serde(default)]
    /// Generate a coverage report for the executed tests which have passed and are not fuzz tests
    pub coverage: bool,
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

    forks
        .iter()
        .try_for_each(|fork| match fork.block_id.len() {
            1 => Ok(()),
            _ => bail!("block_id should be set once per fork"),
        })?;

    Ok(raw_config)
}

impl TryFrom<RawForgeConfig> for ForgeConfigFromScarb {
    type Error = anyhow::Error;

    fn try_from(value: RawForgeConfig) -> Result<Self, Self::Error> {
        let value = validate_raw_fork_config(value)?;
        let mut fork_targets = vec![];

        for raw_fork_target in value.fork {
            let (block_id_type, block_id_value) =
                raw_fork_target.block_id.iter().exactly_one().unwrap();

            fork_targets.push(ForkTarget::new(
                raw_fork_target.name.as_str(),
                raw_fork_target.url.as_str(),
                block_id_type,
                block_id_value,
            )?);
        }

        Ok(ForgeConfigFromScarb {
            exit_first: value.exit_first,
            fuzzer_runs: value.fuzzer_runs,
            fuzzer_seed: value.fuzzer_seed,
            detailed_resources: value.detailed_resources,
            save_trace_data: value.save_trace_data,
            build_profile: value.build_profile,
            coverage: value.coverage,
            fork: fork_targets,
            max_n_steps: value.max_n_steps,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;
    use url::Url;

    #[test]
    fn test_fork_target_new_valid_number() {
        let name = "TestFork";
        let url = "http://example.com";
        let block_id_type = "number";
        let block_id_value = "123";

        let fork_target = ForkTarget::new(name, url, block_id_type, block_id_value).unwrap();

        assert_eq!(fork_target.name, name);
        assert_eq!(fork_target.url, Url::parse(url).unwrap());
        if let BlockId::BlockNumber(number) = fork_target.block_id {
            assert_eq!(number, 123);
        } else {
            panic!("Expected BlockId::BlockNumber");
        }
    }

    #[test]
    fn test_fork_target_new_valid_hash() {
        let name = "TestFork";
        let url = "http://example.com";
        let block_id_type = "hash";
        let block_id_value = "0x1";

        let fork_target = ForkTarget::new(name, url, block_id_type, block_id_value).unwrap();

        assert_eq!(fork_target.name, name);
        assert_eq!(fork_target.url, Url::parse(url).unwrap());
        if let BlockId::BlockHash(hash) = fork_target.block_id {
            assert_eq!(hash.to_bigint(), BigInt::from(1));
        } else {
            panic!("Expected BlockId::BlockHash");
        }
    }

    #[test]
    fn test_fork_target_new_valid_tag() {
        let name = "TestFork";
        let url = "http://example.com";
        let block_id_type = "tag";
        let block_id_value = "latest";

        let fork_target = ForkTarget::new(name, url, block_id_type, block_id_value).unwrap();

        assert_eq!(fork_target.name, name);
        assert_eq!(fork_target.url, Url::parse(url).unwrap());
        if let BlockId::BlockTag = fork_target.block_id {
            // Expected variant
        } else {
            panic!("Expected BlockId::BlockTag");
        }
    }

    #[test]
    fn test_fork_target_new_invalid_url() {
        let name = "TestFork";
        let url = "invalid_url";
        let block_id_type = "number";
        let block_id_value = "123";

        let result = ForkTarget::new(name, url, block_id_type, block_id_value);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Failed to parse fork url");
    }

    #[test]
    fn test_fork_target_new_invalid_block_id_value_number() {
        let name = "TestFork";
        let url = "http://example.com";
        let block_id_type = "number";
        let block_id_value = "invalid_number";

        let result = ForkTarget::new(name, url, block_id_type, block_id_value);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Failed to parse block number"
        );
    }

    #[test]
    fn test_fork_target_new_invalid_block_id_value_hash() {
        let name = "TestFork";
        let url = "http://example.com";
        let block_id_type = "hash";
        let block_id_value = "invalid_hash";

        let result = ForkTarget::new(name, url, block_id_type, block_id_value);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Failed to parse block hash"
        );
    }
}
