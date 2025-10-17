use num_traits::Pow;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct GasStats {
    pub min: u64,
    pub max: u64,
    pub mean: f64,
    pub std_deviation: f64,
}

impl GasStats {
    #[must_use]
    pub fn new(gas_usages: &[u64]) -> Self {
        let mean = mean(gas_usages);
        Self {
            min: *gas_usages.iter().min().unwrap(),
            max: *gas_usages.iter().max().unwrap(),
            mean,
            std_deviation: std_deviation(mean, gas_usages),
        }
    }
}

#[expect(clippy::cast_precision_loss)]
fn mean(values: &[u64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let sum: f64 = values.iter().map(|&x| x as f64).sum();
    sum / values.len() as f64
}

#[expect(clippy::cast_precision_loss)]
fn std_deviation(mean: f64, values: &[u64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let sum_squared_diff = values
        .iter()
        .map(|&x| (x as f64 - mean).pow(2))
        .sum::<f64>();

    (sum_squared_diff / values.len() as f64).sqrt()
}

#[cfg(test)]
mod tests {
    use super::{mean, std_deviation};

    const FLOAT_ERROR: f64 = 0.01;

    #[test]
    fn test_mean_basic() {
        let data = [1, 2, 3, 4, 5];
        let result = mean(&data);
        assert!((result - 3.0).abs() < FLOAT_ERROR);
    }

    #[test]
    fn test_mean_single_element() {
        let data = [42];
        let result = mean(&data);
        assert!((result - 42.0).abs() < FLOAT_ERROR);
    }

    #[test]
    fn test_std_deviation_basic() {
        let data = [1, 2, 3, 4, 5];
        let mean_value = mean(&data);
        let result = std_deviation(mean_value, &data);
        assert!((result - 1.414).abs() < FLOAT_ERROR);
    }

    #[test]
    fn test_std_deviation_single_element() {
        let data = [10];
        let mean_value = mean(&data);
        let result = std_deviation(mean_value, &data);
        assert!(result.abs() < FLOAT_ERROR);
    }
}
