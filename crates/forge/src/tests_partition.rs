use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub struct TestPartition {
    index: usize,
    total: usize,
}

impl TestPartition {
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

impl FromStr for TestPartition {
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

        Ok(TestPartition { index, total })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_happy_case() {
        let partition = "2/5".parse::<TestPartition>().unwrap();
        assert_eq!(partition.index_1_based(), 2);
        assert_eq!(partition.index_0_based(), 1);
        assert_eq!(partition.total(), 5);
    }

    #[test]
    fn test_invalid_format() {
        let err = "2-5".parse::<TestPartition>().unwrap_err();
        assert_eq!(err, "Partition must be in the format <INDEX>/<TOTAL>");
    }

    #[test]
    fn test_non_integer() {
        let err = "a/5".parse::<TestPartition>().unwrap_err();
        assert_eq!(err, "INDEX must be a positive integer");

        let err = "2/b".parse::<TestPartition>().unwrap_err();
        assert_eq!(err, "TOTAL must be a positive integer");
    }

    #[test]
    fn test_out_of_bounds() {
        let err = "0/5".parse::<TestPartition>().unwrap_err();
        assert_eq!(err, "Invalid partition values: ensure 1 <= INDEX <= TOTAL");

        let err = "6/5".parse::<TestPartition>().unwrap_err();
        assert_eq!(err, "Invalid partition values: ensure 1 <= INDEX <= TOTAL");

        let err = "2/0".parse::<TestPartition>().unwrap_err();
        assert_eq!(err, "Invalid partition values: ensure 1 <= INDEX <= TOTAL");
    }
}
