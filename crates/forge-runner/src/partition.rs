use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::str::FromStr;
use std::sync::Arc;

use crate::package_tests::raw::TestTargetRaw;
use crate::package_tests::with_config_resolved::sanitize_test_case_name;
use crate::scarb::load_test_artifacts;
use anyhow::{Result, anyhow};
use camino::Utf8Path;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use scarb_api::metadata::PackageMetadata;
use serde::Serialize;

/// Enum representing configuration for partitioning test cases.
/// Used only when `--partition` flag is provided.
/// If `Disabled`, partitioning is disabled.
/// If `Enabled`, contains the partition information and map of test case names and their partition numbers.
#[derive(Debug, PartialEq, Clone, Default)]
pub enum PartitionConfig {
    #[default]
    Disabled,
    Enabled {
        partition: Partition,
        partition_map: Arc<PartitionMap>,
    },
}

impl PartitionConfig {
    pub fn new(
        partition: Partition,
        packages: &[PackageMetadata],
        artifacts_dir: &Utf8Path,
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
    /// The 1-based index of the partition to run.
    index: NonZeroUsize,
    /// The total number of partitions in the run.
    total: NonZeroUsize,
}

impl Partition {
    #[must_use]
    pub fn index(&self) -> NonZeroUsize {
        self.index
    }

    #[must_use]
    pub fn total(&self) -> NonZeroUsize {
        self.total
    }
}

impl FromStr for Partition {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let (index_str, total_str) = s
            .split_once('/')
            .ok_or_else(|| "Partition must be in the format <INDEX>/<TOTAL>".to_string())?;

        if total_str.contains('/') {
            return Err("Partition must be in the format <INDEX>/<TOTAL>".to_string());
        }

        let index = index_str
            .parse::<NonZeroUsize>()
            .map_err(|_| "INDEX must be a positive integer".to_string())?;
        let total = total_str
            .parse::<NonZeroUsize>()
            .map_err(|_| "TOTAL must be a positive integer".to_string())?;

        if index > total {
            return Err("Invalid partition values: ensure 1 <= INDEX <= TOTAL".to_string());
        }

        Ok(Partition { index, total })
    }
}

/// Map containing test full paths and their assigned partition numbers.
#[derive(Serialize, Debug, PartialEq)]
pub struct PartitionMap(HashMap<String, NonZeroUsize>);

impl PartitionMap {
    /// Builds a partition map from the provided packages and artifacts directory.
    /// Test full paths are sorted to ensure consistent assignment across runs.
    /// Each test case is assigned to a partition with round-robin.
    pub fn build(
        packages: &[PackageMetadata],
        artifacts_dir: &Utf8Path,
        total_partitions: NonZeroUsize,
    ) -> Result<Self> {
        let mut test_full_paths: Vec<String> = packages
            .iter()
            .map(|package| -> Result<Vec<String>> {
                let raw_test_targets = load_test_artifacts(artifacts_dir, package)?;

                let full_paths: Vec<String> = raw_test_targets
                    .iter()
                    .map(collect_test_full_paths)
                    .collect::<Result<Vec<Vec<String>>>>()?
                    .into_iter()
                    .flatten()
                    .collect();

                Ok(full_paths)
            })
            .collect::<Result<Vec<Vec<String>>>>()?
            .into_iter()
            .flatten()
            .collect();

        test_full_paths.sort();

        let mut partition_map = HashMap::with_capacity(test_full_paths.len());

        for (i, test_full_path) in test_full_paths.iter().enumerate() {
            let sanitized_full_path = sanitize_test_case_name(test_full_path);
            let partition_index_1_based = (i % total_partitions) + 1;
            let partition_index_1_based = NonZeroUsize::try_from(partition_index_1_based)?;
            if partition_map
                .insert(sanitized_full_path, partition_index_1_based)
                .is_some()
            {
                unreachable!("Test case full path should be unique");
            }
        }

        Ok(Self(partition_map))
    }

    #[must_use]
    pub fn get_assigned_index(&self, test_full_path: &str) -> Option<NonZeroUsize> {
        self.0.get(test_full_path).copied()
    }
}

/// Collects test full paths from a raw test target.
fn collect_test_full_paths(test_target_raw: &TestTargetRaw) -> Result<Vec<String>> {
    let executables = test_target_raw
        .sierra_program
        .debug_info
        .as_ref()
        .and_then(|info| info.executables.get("snforge_internal_test_executable"))
        .map(Vec::as_slice)
        .unwrap_or_default();
    executables
        .par_iter()
        .map(|case| {
            case.debug_name
                .clone()
                .map(Into::into)
                .ok_or_else(|| anyhow!("Missing debug name for test executable entry"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("1/1", 1, 1)]
    #[test_case("2/5", 2, 5)]
    fn test_happy_case(partition: &str, expected_index: usize, expected_total: usize) {
        let partition = partition.parse::<Partition>().unwrap();
        assert_eq!(
            partition.index,
            NonZeroUsize::try_from(expected_index).unwrap()
        );
        assert_eq!(
            partition.total,
            NonZeroUsize::try_from(expected_total).unwrap()
        );
    }

    #[test_case("1-2"; "using hyphen instead of slash")]
    #[test_case("1/2/3"; "too many parts")]
    #[test_case("12"; "no separator")]
    fn test_invalid_format(partition: &str) {
        let err = partition.parse::<Partition>().unwrap_err();
        assert_eq!(err, "Partition must be in the format <INDEX>/<TOTAL>");
    }

    #[test_case("-1/5", "INDEX")]
    #[test_case("2/-5", "TOTAL")]
    #[test_case("a/5", "INDEX")]
    #[test_case("2/b", "TOTAL")]
    fn test_invalid_integer(partition: &str, invalid_part: &str) {
        let err = partition.parse::<Partition>().unwrap_err();
        assert_eq!(err, format!("{invalid_part} must be a positive integer"));
    }

    #[test_case("0/5")]
    #[test_case("6/5")]
    #[test_case("2/0")]
    fn test_out_of_bounds(partition: &str) {
        assert!(partition.parse::<Partition>().is_err());
    }
}
