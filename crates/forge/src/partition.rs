use serde::Serialize;
use std::str::FromStr;

/// Represents a specific partition in a partitioned test run.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct Partition {
    /// The 1-based index of the partition to run.
    index: usize,
    /// The total number of partitions in the run.
    total: usize,
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("1/1", 1, 1)]
    #[test_case("2/5", 2, 5)]
    fn test_happy_case(partition: &str, expected_index: usize, expected_total: usize) {
        let partition = partition.parse::<Partition>().unwrap();
        assert_eq!(partition.index, expected_index);
        assert_eq!(partition.total, expected_total);
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
        let err = partition.parse::<Partition>().unwrap_err();
        assert_eq!(err, "Invalid partition values: ensure 1 <= INDEX <= TOTAL");
    }
}
