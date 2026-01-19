use serde::Serialize;
use std::str::FromStr;

/// Represents a specific partition in a partitioned test run.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct Partition {
    index: usize,
    total: usize,
}

impl Partition {
    #[must_use]
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

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test]
    fn test_happy_case() {
        let partition = "2/5".parse::<Partition>().unwrap();
        assert_eq!(partition.index, 2);
        assert_eq!(partition.total, 5);
    }

    #[test]
    fn test_invalid_format() {
        let err = "2-5".parse::<Partition>().unwrap_err();
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
