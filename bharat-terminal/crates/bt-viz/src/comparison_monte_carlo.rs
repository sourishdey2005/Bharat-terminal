// crates/bt-viz/src/comparison_monte_carlo.rs
// Author: Sourish Dey

//! Monte Carlo comparison. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct ComparisonMonteCarloConfig {
    pub title: String,
    pub theme: Theme,
    pub simulations: usize,
    pub days: usize,
}

impl Default for ComparisonMonteCarloConfig {
    fn default() -> Self {
        Self {
            title: "Monte Carlo Comparison".to_string(),
            theme: Theme::Dark,
            simulations: 100,
            days: 252,
        }
    }
}

impl ComparisonMonteCarloConfig {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
    pub fn simulations(mut self, v: usize) -> Self {
        self.simulations = v.clamp(10, 1000);
        self
    }
    pub fn days(mut self, v: usize) -> Self {
        self.days = v.max(10);
        self
    }
}

fn monte_carlo_paths(
    series: &OhlcvSeries,
    n_sims: usize,
    n_days: usize,
    seed: u64,
) -> Vec<Vec<f64>> {
    let returns = series.returns();
    let mean = returns.iter().sum::<f64>() / returns.len().max(1) as f64;
    let variance =
        returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / returns.len().max(1) as f64;
    let std_dev = variance.sqrt();
    let start_price = series.candles.last().unwrap().close;

    let mut rng = seed;
    let mut all_paths = Vec::with_capacity(n_sims);

    for _ in 0..n_sims {
        let mut path = Vec::with_capacity(n_days);
        let mut price = start_price;
        for _ in 0..n_days {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let u1 = ((rng >> 11) as f64) / (1u64 << 53) as f64;
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let u2 = ((rng >> 11) as f64) / (1u64 << 53) as f64;
            let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            price *= 1.0 + mean + std_dev * z;
            path.push(price);
        }
        all_paths.push(path);
    }

    all_paths
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series_a: &OhlcvSeries,
    series_b: &OhlcvSeries,
    cfg: &ComparisonMonteCarloConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series_a.validate()?;
    series_b.validate()?;
    fill_background(&root, cfg.theme)?;

    let paths_a = monte_carlo_paths(series_a, cfg.simulations, cfg.days, 1);
    let paths_b = monte_carlo_paths(series_b, cfg.simulations, cfg.days, 2);

    let t_min = 0.0;
    let t_max = cfg.days as f64;
    let y_min = paths_a
        .iter()
        .chain(paths_b.iter())
        .flat_map(|p| p.iter().copied())
        .fold(f64::MAX, f64::min)
        * 0.9;
    let y_max = paths_a
        .iter()
        .chain(paths_b.iter())
        .flat_map(|p| p.iter().copied())
        .fold(f64::MIN, f64::max)
        * 1.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!(
                "{} — {} vs {} ({} sims)",
                cfg.title, series_a.symbol, series_b.symbol, cfg.simulations
            ),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(t_min..t_max, y_min..y_max)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    for path in &paths_a {
        chart
            .draw_series(LineSeries::new(
                path.iter().enumerate().map(|(i, &p)| (i as f64, p)),
                cfg.theme.profit().mix(0.1).stroke_width(1),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    for path in &paths_b {
        chart
            .draw_series(LineSeries::new(
                path.iter().enumerate().map(|(i, &p)| (i as f64, p)),
                cfg.theme.loss().mix(0.1).stroke_width(1),
            ))
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(
    series_a: &OhlcvSeries,
    series_b: &OhlcvSeries,
    cfg: &ComparisonMonteCarloConfig,
    path: &str,
) -> Result<()> {
    render(png_root(path)?, series_a, series_b, cfg)
}

pub fn render_svg(
    series_a: &OhlcvSeries,
    series_b: &OhlcvSeries,
    cfg: &ComparisonMonteCarloConfig,
    path: &str,
) -> Result<()> {
    render(svg_root(path)?, series_a, series_b, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let a = synthetic_ohlcv("AAA", 100, 1, 100.0);
        let b = synthetic_ohlcv("BBB", 100, 2, 100.0);
        let cfg = ComparisonMonteCarloConfig::new()
            .simulations(20)
            .theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_comparison_monte_carlo.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&a, &b, &cfg, &path).unwrap();
    }
}
