use crate::run_tests::package::RunForPackageArgs;
use cairo_lang_sierra::ids::FunctionId;
use forge_runner::package_tests::with_config_resolved::sanitize_test_case_name;
use serde::Serialize;
use std::{collections::HashMap, str::FromStr};

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Partition {
    index: usize,
    total: usize,
}

impl Partition {
    #[must_use]
    pub fn index_0_based(&self) -> usize {
        self.index - 1
    }

    #[must_use]
    pub fn index_1_based(&self) -> usize {
        self.index
    }

    #[must_use]
    pub fn total(&self) -> usize {
        self.total
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

#[derive(Serialize)]
pub struct TestsPartitionsMapping(HashMap<String, usize>);

impl TestsPartitionsMapping {
    pub fn get(&self, test_name: &str) -> Option<&usize> {
        self.0.get(test_name)
    }

    pub fn insert(&mut self, test_name: String, partition_index: usize) {
        self.0.insert(test_name, partition_index);
    }

    pub fn from_packages_args(packages_args: &[RunForPackageArgs], partition: Partition) -> Self {
        let mut full_paths: Vec<String> = packages_args
            .iter()
            .flat_map(|pkg| pkg.test_targets.iter())
            .flat_map(|tt| {
                tt.sierra_program
                    .debug_info
                    .as_ref()
                    .and_then(|info| info.executables.get("snforge_internal_test_executable"))
                    .into_iter()
                    .flatten()
            })
            .filter_map(|fid: &FunctionId| {
                fid.debug_name
                    .as_ref()
                    .map(std::string::ToString::to_string)
            })
            .collect();
        // println!("names before sort: {:?}", full_paths);
        // for name in &full_paths {
        //     println!("name: {}", name);
        // }
        full_paths.sort();

        // println!("names after sort: {:?}", full_paths);
        // for name in &full_paths {
        //     println!("name: {}", name);
        // }
        let total = partition.total();
        let mut mapping = HashMap::with_capacity(full_paths.len());

        for (i, path) in full_paths.into_iter().enumerate() {
            let partition_index_1_based = (i % total) + 1;
            mapping.insert(sanitize_test_case_name(&path), partition_index_1_based);
        }

        Self(mapping)
    }
}

#[derive(Serialize)]
pub struct PartitionConfig {
    partition: Partition,
    partitions_mapping: TestsPartitionsMapping,
}

impl PartitionConfig {
    pub fn new(partition: Partition, packages_args: &[RunForPackageArgs]) -> Self {
        let partitions_mapping =
            TestsPartitionsMapping::from_packages_args(packages_args, partition);
        Self {
            partition,
            partitions_mapping,
        }
    }

    pub fn partition(&self) -> Partition {
        self.partition
    }

    pub fn partitions_mapping(&self) -> &TestsPartitionsMapping {
        &self.partitions_mapping
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_happy_case() {
        let partition = "2/5".parse::<Partition>().unwrap();
        assert_eq!(partition.index_1_based(), 2);
        assert_eq!(partition.index_0_based(), 1);
        assert_eq!(partition.total(), 5);
    }

    #[test]
    fn test_invalid_format() {
        let err = "2-5".parse::<Partition>().unwrap_err();
        assert_eq!(err, "Partition must be in the format <INDEX>/<TOTAL>");
    }

    #[test]
    fn test_non_integer() {
        let err = "a/5".parse::<Partition>().unwrap_err();
        assert_eq!(err, "INDEX must be a positive integer");

        let err = "2/b".parse::<Partition>().unwrap_err();
        assert_eq!(err, "TOTAL must be a positive integer");
    }

    #[test]
    fn test_out_of_bounds() {
        let err = "0/5".parse::<Partition>().unwrap_err();
        assert_eq!(err, "Invalid partition values: ensure 1 <= INDEX <= TOTAL");

        let err = "6/5".parse::<Partition>().unwrap_err();
        assert_eq!(err, "Invalid partition values: ensure 1 <= INDEX <= TOTAL");

        let err = "2/0".parse::<Partition>().unwrap_err();
        assert_eq!(err, "Invalid partition values: ensure 1 <= INDEX <= TOTAL");
    }
}
