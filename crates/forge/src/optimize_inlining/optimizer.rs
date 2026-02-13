use crate::optimize_inlining::args::OptimizeInliningArgs;
use crate::optimize_inlining::runner::{OptimizationResult, TotalGas, run_optimization_iteration};
use anyhow::{Result, anyhow};
use camino::Utf8Path;
use foundry_ui::UI;
use plotters::prelude::*;
use scarb_api::metadata::Metadata;
use std::sync::Arc;

pub struct Optimizer {
    pub min_threshold: u32,
    pub max_threshold: u32,
    pub step: u32,
    pub results: Vec<OptimizationResult>,
    scarb_metadata: Metadata,
}

pub struct OptimalResult {
    pub threshold: u32,
    pub total_gas: TotalGas,
    pub max_contract_size: u64,
    pub max_contract_felts: u64,
}

impl Optimizer {
    pub fn new(args: &OptimizeInliningArgs, scarb_metadata: &Metadata) -> Self {
        Self {
            min_threshold: args.min_threshold,
            max_threshold: args.max_threshold,
            step: args.step,
            results: Vec::new(),
            scarb_metadata: scarb_metadata.clone(),
        }
    }

    fn run_boundary_tests(&mut self, args: &OptimizeInliningArgs, ui: &Arc<UI>) -> Result<()> {
        ui.println(&"Running boundary tests...".to_string());

        ui.print_blank_line();
        ui.println(&format!("Testing min threshold {}...", self.min_threshold));
        let min_result =
            run_optimization_iteration(self.min_threshold, args, &self.scarb_metadata, ui)?;
        if min_result.tests_passed && min_result.error.is_none() {
            ui.println(&format!(
                "  ✓ gas: {}, felts: {}",
                min_result.total_gas.total(),
                min_result.max_contract_felts
            ));
        } else {
            ui.println(&format!(
                "  ✗ {}",
                min_result.error.as_deref().unwrap_or("failed")
            ));
        }

        ui.print_blank_line();
        ui.println(&format!("Testing max threshold {}...", self.max_threshold));
        let max_result =
            run_optimization_iteration(self.max_threshold, args, &self.scarb_metadata, ui)?;
        if max_result.tests_passed && max_result.error.is_none() {
            ui.println(&format!(
                "  ✓ gas: {}, felts: {}",
                max_result.total_gas.total(),
                max_result.max_contract_felts
            ));
        } else {
            ui.println(&format!(
                "  ✗ {}",
                max_result.error.as_deref().unwrap_or("failed")
            ));
        }

        self.results.push(min_result);
        self.results.push(max_result);

        Ok(())
    }

    fn get_max_values(&self) -> (u64, u64) {
        let valid_results: Vec<_> = self
            .results
            .iter()
            .filter(|r| r.tests_passed && r.error.is_none())
            .collect();

        let max_gas = valid_results
            .iter()
            .map(|r| r.total_gas.total())
            .max()
            .unwrap_or(1);
        let max_felts = valid_results
            .iter()
            .map(|r| r.max_contract_felts)
            .max()
            .unwrap_or(1);

        (max_gas, max_felts)
    }

    pub fn optimize(&mut self, args: &OptimizeInliningArgs, ui: &Arc<UI>) -> Result<OptimalResult> {
        self.run_boundary_tests(args, ui)?;
        self.optimize_bruteforce(args, ui)
    }

    fn optimize_bruteforce(
        &mut self,
        args: &OptimizeInliningArgs,
        ui: &Arc<UI>,
    ) -> Result<OptimalResult> {
        let total_iterations = ((self.max_threshold - self.min_threshold) / self.step) + 1;
        let mut current = self.min_threshold;
        let mut iteration = 1;

        while current <= self.max_threshold {
            if !self.results.iter().any(|r| r.threshold == current) {
                ui.print_blank_line();
                ui.println(&format!(
                    "[{}/{}] Testing threshold {}...",
                    iteration, total_iterations, current
                ));

                let result = run_optimization_iteration(current, args, &self.scarb_metadata, ui)?;

                if let Some(ref error) = result.error {
                    ui.println(&format!("  ✗ {}", error));
                } else {
                    ui.println(&format!(
                        "  ✓ Tests passed, gas: {}, max contract size: {} bytes, max felts: {}",
                        result.total_gas.total(),
                        result.max_contract_size,
                        result.max_contract_felts
                    ));
                }

                self.results.push(result);
            }
            current += self.step;
            iteration += 1;
        }

        self.find_best_result()
    }

    fn find_best_result(&self) -> Result<OptimalResult> {
        self.results
            .iter()
            .filter(|r| r.tests_passed && r.error.is_none())
            .min_by(|a, b| {
                a.total_gas
                    .total()
                    .cmp(&b.total_gas.total())
                    .then(a.max_contract_felts.cmp(&b.max_contract_felts))
                    .then(a.threshold.cmp(&b.threshold))
            })
            .map(|r| OptimalResult {
                threshold: r.threshold,
                total_gas: r.total_gas.clone(),
                max_contract_size: r.max_contract_size,
                max_contract_felts: r.max_contract_felts,
            })
            .ok_or_else(|| anyhow!("No valid optimization results found"))
    }

    pub fn print_results_table(&self, ui: &UI) {
        let mut sorted_results: Vec<_> = self.results.iter().collect();
        sorted_results.sort_by_key(|r| r.threshold);

        ui.println(
            &"┌──────────────┬─────────────────┬──────────────────┬──────────────┬────────┐"
                .to_string(),
        );
        ui.println(
            &"│  Threshold   │    Total Gas    │  Contract Size   │    Felts     │ Status │"
                .to_string(),
        );
        ui.println(
            &"├──────────────┼─────────────────┼──────────────────┼──────────────┼────────┤"
                .to_string(),
        );

        for r in sorted_results {
            let status = if r.tests_passed && r.error.is_none() {
                "✓"
            } else {
                "✗"
            };
            let gas_str = if r.tests_passed && r.error.is_none() {
                format!("{:>13}", r.total_gas.total())
            } else {
                format!("{:>13}", "-")
            };
            ui.println(&format!(
                "│ {:>10}   │ {}   │ {:>14}   │ {:>10}   │   {}    │",
                r.threshold, gas_str, r.max_contract_size, r.max_contract_felts, status
            ));
        }

        ui.println(
            &"└──────────────┴─────────────────┴──────────────────┴──────────────┴────────┘"
                .to_string(),
        );

        if let Ok(best) = self.find_best_result() {
            ui.print_blank_line();
            ui.println(&format!(
                "Best result: threshold={}, gas={}, felts={}",
                best.threshold,
                best.total_gas.total(),
                best.max_contract_felts
            ));
        }
    }

    pub fn save_results_graph(&self, output_path: &Utf8Path, ui: &UI) -> Result<()> {
        let mut sorted_results: Vec<_> = self
            .results
            .iter()
            .filter(|r| r.tests_passed && r.error.is_none())
            .collect();

        if sorted_results.len() < 2 {
            ui.println(&"Not enough data points to draw graph (need at least 2)".to_string());
            return Ok(());
        }

        sorted_results.sort_by_key(|r| r.threshold);

        let (max_gas, max_felts) = self.get_max_values();

        let gas_points: Vec<(f64, f64)> = sorted_results
            .iter()
            .map(|r| {
                (
                    r.threshold as f64,
                    r.total_gas.total() as f64 / max_gas as f64,
                )
            })
            .collect();

        let felts_points: Vec<(f64, f64)> = sorted_results
            .iter()
            .map(|r| {
                (
                    r.threshold as f64,
                    r.max_contract_felts as f64 / max_felts as f64,
                )
            })
            .collect();

        let x_min = self.min_threshold as f64;
        let x_max = self.max_threshold as f64;

        let y_min = gas_points
            .iter()
            .chain(felts_points.iter())
            .map(|(_, y)| *y)
            .fold(f64::MAX, f64::min)
            * 0.95;
        let y_max = gas_points
            .iter()
            .chain(felts_points.iter())
            .map(|(_, y)| *y)
            .fold(f64::MIN, f64::max)
            * 1.05;

        let root = BitMapBackend::new(output_path.as_std_path(), (1920, 1080)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption("Inlining Optimization Results", ("sans-serif", 48))
            .margin(40)
            .x_label_area_size(80)
            .y_label_area_size(100)
            .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

        chart
            .configure_mesh()
            .x_desc("Threshold")
            .y_desc("Normalized Value")
            .x_label_formatter(&|x| format!("{:.0}", x))
            .y_label_formatter(&|y| format!("{:.2}", y))
            .draw()?;

        chart
            .draw_series(LineSeries::new(gas_points.clone(), RED.stroke_width(3)))?
            .label("Gas")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 40, y)], RED.stroke_width(3)));

        chart.draw_series(
            gas_points
                .iter()
                .map(|(x, y)| Circle::new((*x, *y), 8, RED.filled())),
        )?;

        chart
            .draw_series(LineSeries::new(felts_points.clone(), GREEN.stroke_width(3)))?
            .label("Weight")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 40, y)], GREEN.stroke_width(3)));

        chart.draw_series(
            felts_points
                .iter()
                .map(|(x, y)| Circle::new((*x, *y), 8, GREEN.filled())),
        )?;

        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .position(SeriesLabelPosition::UpperRight)
            .draw()?;

        root.present()?;

        ui.print_blank_line();
        ui.println(&format!("Graph saved to: {}", output_path));

        Ok(())
    }
}
