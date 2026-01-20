use serde::Serialize;
use std::{num::NonZeroUsize, str::FromStr};

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

        println!(
            "Parsing partition: index_str = {}, total_str = {}",
            index_str, total_str
        );
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
