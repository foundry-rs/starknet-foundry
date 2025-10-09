use crate::run_tests::package::RunForPackageArgs;
use cairo_lang_sierra::ids::FunctionId;
use std::{collections::HashMap, str::FromStr};

#[derive(Debug, Clone, Copy)]
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

pub type TestsPartitionsMapping = HashMap<String, usize>;

fn partitions_mapping_from_packages_args(
    packages_args: &[RunForPackageArgs],
    partition: Partition,
) -> TestsPartitionsMapping {
    let names: Vec<String> = packages_args
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

    let total = partition.total();
    let mut mapping = HashMap::with_capacity(names.len());

    for (i, name) in names.into_iter().enumerate() {
        let shard_1_based = (i % total) + 1;
        mapping.insert(name, shard_1_based);
    }

    mapping
}

pub struct PartitionConfig {
    partition: Partition,
    partitions_mapping: TestsPartitionsMapping,
}

impl PartitionConfig {
    pub fn new(partition: Partition, packages_args: &[RunForPackageArgs]) -> Self {
        let partitions_mapping = partitions_mapping_from_packages_args(packages_args, partition);
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
