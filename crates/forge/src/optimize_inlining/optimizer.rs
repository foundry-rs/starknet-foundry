use crate::optimize_inlining::args::OptimizeInliningArgs;
use crate::optimize_inlining::runner::{
    OptimizationIterationResult, TotalGas, run_optimization_iteration,
};
use anyhow::{Result, anyhow};
use camino::Utf8Path;
use foundry_ui::UI;
use plotters::prelude::*;
use plotters::style::{FontStyle, register_font};
use scarb_api::metadata::Metadata;
use std::sync::Arc;

const ROBOTO_REGULAR: &[u8] = include_bytes!("../../assets/fonts/Roboto-Regular.ttf");
const ROBOTO_FAMILY: &str = "roboto";

pub struct Optimizer {
    pub min_threshold: u32,
    pub max_threshold: u32,
    pub step: u32,
    pub results: Vec<OptimizationIterationResult>,
    scarb_metadata: Metadata,
}

pub struct OptimalResult {
    pub threshold: u32,
    pub total_gas: TotalGas,
    pub contract_code_l2_gas: u64,
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

    fn run_boundary_tests(
        &mut self,
        args: &OptimizeInliningArgs,
        cores: usize,
        ui: &Arc<UI>,
    ) -> Result<()> {
        ui.println(&"Running boundary tests...".to_string());

        for (label, threshold) in [("min", self.min_threshold), ("max", self.max_threshold)] {
            ui.print_blank_line();
            ui.println(&format!("Testing {label} threshold {threshold}..."));
            let result =
                run_optimization_iteration(threshold, args, &self.scarb_metadata, cores, ui)?;
            if result.tests_passed && result.error.is_none() {
                ui.println(&format!(
                    "  ✓ gas: {:.0}, contract bytecode L2 gas: {}",
                    result.total_gas.l2(),
                    result.contract_code_l2_gas
                ));
            } else {
                ui.println(&format!(
                    "  ✗ {}",
                    result.error.as_deref().unwrap_or("failed")
                ));
            }
            self.results.push(result);
        }

        Ok(())
    }

    fn valid_results(&self) -> Result<Vec<&OptimizationIterationResult>> {
        let results: Vec<_> = self
            .results
            .iter()
            .filter(|r| r.tests_passed && r.error.is_none())
            .collect();
        if results.is_empty() {
            return Err(anyhow!("No valid optimization results found"));
        }
        Ok(results)
    }

    fn get_max_values(&self) -> (f64, f64) {
        let valid_results = self.valid_results().unwrap_or_default();

        let max_gas = valid_results
            .iter()
            .map(|r| r.total_gas.l2())
            .fold(1.0_f64, f64::max);
        let max_contract_code_l2_gas = valid_results
            .iter()
            .map(|r| r.contract_code_l2_gas)
            .max()
            .unwrap_or(1);

        #[allow(clippy::cast_precision_loss)]
        (max_gas, max_contract_code_l2_gas as f64)
    }

    pub fn optimize(
        &mut self,
        args: &OptimizeInliningArgs,
        cores: usize,
        ui: &Arc<UI>,
    ) -> Result<OptimalResult> {
        self.run_boundary_tests(args, cores, ui)?;
        self.optimize_bruteforce(args, cores, ui)
    }

    fn optimize_bruteforce(
        &mut self,
        args: &OptimizeInliningArgs,
        cores: usize,
        ui: &Arc<UI>,
    ) -> Result<OptimalResult> {
        let total_iterations = ((self.max_threshold - self.min_threshold) / self.step) + 1;
        let mut current = self.min_threshold;
        let mut iteration = 1;

        while current <= self.max_threshold {
            if !self.results.iter().any(|r| r.threshold == current) {
                ui.print_blank_line();
                ui.println(&format!(
                    "[{iteration}/{total_iterations}] Testing threshold {current}...",
                ));

                let result =
                    run_optimization_iteration(current, args, &self.scarb_metadata, cores, ui)?;

                if let Some(ref error) = result.error {
                    ui.println(&format!("  ✗ {error}",));
                } else {
                    ui.println(&format!(
                        "  ✓ Tests passed, gas: {:.0}, max contract size: {} bytes, contract bytecode L2 gas: {}",
                        result.total_gas.l2(),
                        result.max_contract_size,
                        result.contract_code_l2_gas
                    ));
                }

                self.results.push(result);
            }
            current += self.step;
            iteration += 1;
        }

        self.find_best_result_by_gas()
    }

    pub fn find_best_result_by_gas(&self) -> Result<OptimalResult> {
        Ok(self
            .valid_results()?
            .into_iter()
            .min_by(|a, b| {
                a.total_gas
                    .l2()
                    .total_cmp(&b.total_gas.l2())
                    .then(a.contract_code_l2_gas.cmp(&b.contract_code_l2_gas))
                    .then(a.threshold.cmp(&b.threshold))
            })
            .map(|r| OptimalResult {
                threshold: r.threshold,
                total_gas: r.total_gas.clone(),
                contract_code_l2_gas: r.contract_code_l2_gas,
            })
            .expect("valid_results must return at least one result"))
    }

    pub fn find_best_result_by_contract_size(&self) -> Result<OptimalResult> {
        Ok(self
            .valid_results()?
            .into_iter()
            .min_by(|a, b| {
                a.contract_code_l2_gas
                    .cmp(&b.contract_code_l2_gas)
                    .then(a.total_gas.l2().total_cmp(&b.total_gas.l2()))
                    .then(a.threshold.cmp(&b.threshold))
            })
            .map(|r| OptimalResult {
                threshold: r.threshold,
                total_gas: r.total_gas.clone(),
                contract_code_l2_gas: r.contract_code_l2_gas,
            })
            .expect("valid_results must return at least one result"))
    }

    pub fn print_results_table(&self, ui: &UI) {
        let mut sorted_results: Vec<_> = self.results.iter().collect();
        sorted_results.sort_by_key(|r| r.threshold);

        ui.println(
            &"┌──────────────┬─────────────────┬──────────────────┬──────────────────────────┬────────┐"
                .to_string(),
        );
        ui.println(
            &"│  Threshold   │    Total Gas    │  Contract Size   │ Contract Bytecode L2 Gas │ Status │"
                .to_string(),
        );
        ui.println(
            &"├──────────────┼─────────────────┼──────────────────┼──────────────────────────┼────────┤"
                .to_string(),
        );

        for r in sorted_results {
            let status = if r.tests_passed && r.error.is_none() {
                "✓"
            } else {
                "✗"
            };
            let gas_str = if r.tests_passed && r.error.is_none() {
                format!("{:>13.0}", r.total_gas.l2())
            } else {
                format!("{:>13}", "-")
            };
            ui.println(&format!(
                "│ {:>10}   │ {}   │ {:>14}   │ {:>24} │   {}    │",
                r.threshold, gas_str, r.max_contract_size, r.contract_code_l2_gas, status
            ));
        }

        ui.println(
            &"└──────────────┴─────────────────┴──────────────────┴──────────────────────────┴────────┘"
                .to_string(),
        );

        if let Ok(best) = self.find_best_result_by_gas() {
            ui.print_blank_line();
            ui.println(&format!(
                "Lowest runtime gas cost: threshold={}, gas={:.0}, contract bytecode L2 gas={}",
                best.threshold,
                best.total_gas.l2(),
                best.contract_code_l2_gas
            ));
        }
        if let Ok(best) = self.find_best_result_by_contract_size() {
            ui.println(&format!(
                "Lowest contract size cost: threshold={}, gas={:.0}, contract bytecode L2 gas={}",
                best.threshold,
                best.total_gas.l2(),
                best.contract_code_l2_gas
            ));
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn save_results_graph(&self, output_path: &Utf8Path, ui: &UI) -> Result<()> {
        for style in [
            FontStyle::Normal,
            FontStyle::Bold,
            FontStyle::Oblique,
            FontStyle::Italic,
        ] {
            register_font(ROBOTO_FAMILY, style, ROBOTO_REGULAR)
                .map_err(|_| anyhow!("Failed to register bundled Roboto-Regular.ttf font"))?;
            register_font("sans-serif", style, ROBOTO_REGULAR)
                .map_err(|_| anyhow!("Failed to register bundled Roboto-Regular.ttf font"))?;
        }

        let mut sorted_results = self.valid_results()?;

        if sorted_results.len() < 2 {
            ui.println(&"Not enough data points to draw graph (need at least 2)".to_string());
            return Ok(());
        }

        sorted_results.sort_by_key(|r| r.threshold);

        let (max_gas, max_contract_code_l2_gas) = self.get_max_values();

        let gas_points: Vec<(f64, f64)> = sorted_results
            .iter()
            .map(|r| (f64::from(r.threshold), r.total_gas.l2() / max_gas))
            .collect();

        let bytecode_l2_gas_points: Vec<(f64, f64)> = sorted_results
            .iter()
            .map(|r| {
                (f64::from(r.threshold), {
                    #[allow(clippy::cast_precision_loss)]
                    let gas = r.contract_code_l2_gas as f64;
                    gas / max_contract_code_l2_gas
                })
            })
            .collect();

        let x_min = f64::from(self.min_threshold);
        let x_max = f64::from(self.max_threshold);

        let y_min = gas_points
            .iter()
            .chain(bytecode_l2_gas_points.iter())
            .map(|(_, y)| *y)
            .fold(f64::MAX, f64::min)
            * 0.95;
        let y_max = gas_points
            .iter()
            .chain(bytecode_l2_gas_points.iter())
            .map(|(_, y)| *y)
            .fold(f64::MIN, f64::max)
            * 1.05;

        let root = BitMapBackend::new(output_path.as_std_path(), (1920, 1080)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption("Inlining Optimization Results", (ROBOTO_FAMILY, 48))
            .margin(40)
            .x_label_area_size(80)
            .y_label_area_size(100)
            .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

        chart
            .configure_mesh()
            .x_desc("Threshold")
            .y_desc("Normalized Value")
            .label_style((ROBOTO_FAMILY, 24))
            .x_label_formatter(&|x| format!("{x:.0}"))
            .y_label_formatter(&|y| format!("{y:.2}"))
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
            .draw_series(LineSeries::new(
                bytecode_l2_gas_points.clone(),
                GREEN.stroke_width(3),
            ))?
            .label("Contract Bytecode L2 Gas")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 40, y)], GREEN.stroke_width(3)));

        chart.draw_series(
            bytecode_l2_gas_points
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
        ui.println(&format!("Graph saved to: {output_path}"));

        Ok(())
    }
}
