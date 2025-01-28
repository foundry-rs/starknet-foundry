use crate::fuzzer::arguments::CairoType;
use anyhow::{Ok, Result};
use rand::rngs::StdRng;
use rand::Rng;

mod arguments;
mod random;

use std::num::NonZeroU32;

#[derive(Debug, Clone)]
pub struct FuzzerArg {
    cairo_type: CairoType,
    run_with_min_value: u32,
    run_with_max_value: u32,
}

#[derive(Debug, Clone)]
pub struct RunParams {
    /// Arguments
    arguments: Vec<FuzzerArg>,
    /// Total number of runs
    total_runs: NonZeroU32,
    /// Number of already executed runs
    executed_runs: u32,
}

impl RunParams {
    pub fn from(rng: &mut StdRng, total_runs: NonZeroU32, arguments: &[&str]) -> Result<Self> {
        let arguments = arguments
            .iter()
            .map(|arg| -> Result<FuzzerArg> {
                let argument = CairoType::from_name(arg)?;
                if total_runs.get() >= 3 {
                    let run_with_min_value = rng.gen_range(1..=total_runs.get());
                    let run_with_max_value = rng.gen_range(1..=total_runs.get());

                    let run_with_max_value = if run_with_max_value == run_with_min_value {
                        run_with_min_value % total_runs.get() + 1
                    } else {
                        run_with_max_value
                    };

                    Ok(FuzzerArg {
                        cairo_type: argument,
                        run_with_max_value,
                        run_with_min_value,
                    })
                } else {
                    Ok(FuzzerArg {
                        cairo_type: argument,
                        run_with_max_value: u32::MAX,
                        run_with_min_value: u32::MAX,
                    })
                }
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            arguments,
            total_runs,
            executed_runs: 0,
        })
    }
}
