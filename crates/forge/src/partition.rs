use serde::Serialize;
use std::{collections::HashMap, str::FromStr};

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Partition {
    index: usize,
    total: usize,
}

#[allow(dead_code)] // TODO: Removed in later PRs
impl Partition {
    #[must_use]
    pub fn index(&self) -> usize {
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
pub struct TestPartitionMap(HashMap<String, usize>);

#[derive(Serialize)]
pub struct PartitionConfig {
    partition: Partition,
    test_partition_map: TestPartitionMap,
}

#[allow(dead_code)] // TODO: Removed in later PRs
impl PartitionConfig {
    pub fn new(partition: Partition, test_partition_map: TestPartitionMap) -> Self {
        Self {
            partition,
            test_partition_map,
        }
    }

    pub fn partition(&self) -> Partition {
        self.partition
    }

    pub fn test_partition_map(&self) -> &TestPartitionMap {
        &self.test_partition_map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_happy_case() {
        let partition = "2/5".parse::<Partition>().unwrap();
        assert_eq!(partition.index(), 2);
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
