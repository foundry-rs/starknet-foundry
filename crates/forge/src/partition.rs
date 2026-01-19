use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use crate::scarb::load_test_artifacts;
use anyhow::{Result, anyhow};
use camino::Utf8PathBuf;
use forge_runner::package_tests::raw::TestTargetRaw;
use forge_runner::package_tests::with_config_resolved::sanitize_test_case_name;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use scarb_api::metadata::PackageMetadata;
use serde::Serialize;

/// Enum representing configuration for partitioning test cases.
/// If `None`, partitioning is disabled.
/// If `Enabled`, contains the partition information and map of test case names and their partition numbers.
#[derive(Debug, PartialEq, Clone)]
pub enum PartitionConfig {
    None,
    Enabled {
        partition: Partition,
        partition_map: Arc<PartitionMap>,
    },
}

impl PartitionConfig {
    pub fn new(
        partition: Partition,
        packages: &Vec<PackageMetadata>,
        artifacts_dir: &Utf8PathBuf,
    ) -> Result<Self> {
        let partition_map = PartitionMap::build(packages, artifacts_dir, partition.total)?;

        Ok(Self::Enabled {
            partition,
            partition_map: Arc::new(partition_map),
        })
    }
}

/// Represents a specific partition in a partitioned test run.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct Partition {
    index: usize,
    total: usize,
}

impl Partition {
    pub fn index(&self) -> usize {
        self.index
    }
}

impl FromStr for Partition {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();

        if parts.len() != 2 {
            return Err("Partition must be in the format <INDEX>/<TOTAL>".to_string());
        }

        let index = parts[0]
            .parse::<usize>()
            .map_err(|_| "INDEX must be a positive integer".to_string())?;
        let total = parts[1]
            .parse::<usize>()
            .map_err(|_| "TOTAL must be a positive integer".to_string())?;

        if index == 0 || total == 0 || index > total {
            return Err("Invalid partition values: ensure 1 <= INDEX <= TOTAL".to_string());
        }

        Ok(Partition { index, total })
    }
}

/// Map containing test case names and their assigned partition numbers.
#[derive(Serialize, Debug, PartialEq)]
pub struct PartitionMap(HashMap<String, usize>);

impl PartitionMap {
    pub fn build(
        packages: &Vec<PackageMetadata>,
        artifacts_dir: &Utf8PathBuf,
        total_partitions: usize,
    ) -> Result<Self> {
        let mut test_case_names: Vec<String> = packages
            .par_iter()
            .map(|package| -> Result<Vec<String>> {
                let raw_test_targets = load_test_artifacts(artifacts_dir, package)?;

                let names: Vec<String> = raw_test_targets
                    .iter()
                    .map(collect_test_case_names)
                    .collect::<Result<Vec<Vec<String>>>>()?
                    .into_iter()
                    .flatten()
                    .collect();

                Ok(names)
            })
            .collect::<Result<Vec<Vec<String>>>>()?
            .into_iter()
            .flatten()
            .collect();

        test_case_names.sort();

        let mut partition_map = HashMap::with_capacity(test_case_names.len());

        for (i, test_case_name) in test_case_names.iter().enumerate() {
            let partition_index_1_based = (i % total_partitions) + 1;
            partition_map.insert(
                sanitize_test_case_name(&test_case_name),
                partition_index_1_based,
            );
        }

        Ok(Self(partition_map))
    }

    pub fn get_partition_number(&self, test_case_name: &str) -> usize {
        self.0
            .get(test_case_name)
            .expect("Test case name not found in partition map")
            .to_owned()
    }
}

/// Collects test case names from a raw test target.
fn collect_test_case_names(test_target_raw: &TestTargetRaw) -> Result<Vec<String>> {
    let default_executables = vec![];
    let debug_info = test_target_raw.sierra_program.debug_info.clone();
    let executables = debug_info
        .as_ref()
        .and_then(|info| info.executables.get("snforge_internal_test_executable"))
        .unwrap_or(&default_executables);

    let test_case_names: Vec<String> = executables
        .par_iter()
        .map(|case| {
            case.debug_name
                .clone()
                .map(Into::into)
                .ok_or_else(|| anyhow!("Missing debug name for test executable entry"))
        })
        .collect::<Result<Vec<String>>>()?;

    Ok(test_case_names)
}
