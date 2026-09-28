// crates/bt-viz/src/comparison_efficient_frontier.rs
// Author: Sourish Dey

//! Efficient frontier with multiple assets. Made by Sourish Dey.

use bt_analytics::efficient_frontier;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct ComparisonEfficientFrontierConfig {
    pub title: String,
    pub theme: Theme,
    pub n_points: usize,
}

impl Default for ComparisonEfficientFrontierConfig {
    fn default() -> Self {
        Self {
            title: "Efficient Frontier".to_string(),
            theme: Theme::Dark,
            n_points: 20,
        }
    }
}

impl ComparisonEfficientFrontierConfig {
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
    pub fn n_points(mut self, v: usize) -> Self {
        self.n_points = v.clamp(5, 100);
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series_a: &OhlcvSeries,
    series_b: &OhlcvSeries,
    cfg: &ComparisonEfficientFrontierConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series_a.validate()?;
    series_b.validate()?;
    fill_background(&root, cfg.theme)?;

    let returns_a = series_a.returns();
    let returns_b = series_b.returns();

    let n = returns_a.len().min(returns_b.len());
    let mean_a = returns_a.iter().sum::<f64>() / n as f64;
    let mean_b = returns_b.iter().sum::<f64>() / n as f64;

    let var_a = returns_a.iter().map(|r| (r - mean_a).powi(2)).sum::<f64>() / n as f64;
    let var_b = returns_b.iter().map(|r| (r - mean_b).powi(2)).sum::<f64>() / n as f64;
    let cov = returns_a
        .iter()
        .zip(returns_b.iter())
        .map(|(a, b)| (a - mean_a) * (b - mean_b))
        .sum::<f64>()
        / n as f64;

    let expected = vec![mean_a, mean_b];
    let cov_matrix = vec![vec![var_a, cov], vec![cov, var_b]];
    let frontier = efficient_frontier(&expected, &cov_matrix, cfg.n_points);

    let x_min = frontier.iter().map(|(r, _)| *r).fold(f64::MAX, f64::min) * 0.9;
    let x_max = frontier.iter().map(|(r, _)| *r).fold(f64::MIN, f64::max) * 1.1;
    let y_min = frontier.iter().map(|(_, r)| *r).fold(f64::MAX, f64::min) * 0.9;
    let y_max = frontier.iter().map(|(_, r)| *r).fold(f64::MIN, f64::max) * 1.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — {} vs {}", cfg.title, series_a.symbol, series_b.symbol),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(LineSeries::new(
            frontier.iter().map(|(r, ret)| (*r, *ret)),
            cfg.theme.info().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("Efficient Frontier")
        .legend(|(x, y)| {
            PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.info().stroke_width(2))
        });

    chart
        .draw_series(vec![
            Circle::new(
                (var_a.sqrt() * 100.0, mean_a * 100.0),
                5,
                cfg.theme.profit().filled(),
            ),
            Circle::new(
                (var_b.sqrt() * 100.0, mean_b * 100.0),
                5,
                cfg.theme.loss().filled(),
            ),
        ])
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_series_labels()
        .border_style(&cfg.theme.border())
        .background_style(cfg.theme.background().mix(0.8))
        .label_font((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(
    series_a: &OhlcvSeries,
    series_b: &OhlcvSeries,
    cfg: &ComparisonEfficientFrontierConfig,
    path: &str,
) -> Result<()> {
    render(png_root(path)?, series_a, series_b, cfg)
}

pub fn render_svg(
    series_a: &OhlcvSeries,
    series_b: &OhlcvSeries,
    cfg: &ComparisonEfficientFrontierConfig,
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
        let cfg = ComparisonEfficientFrontierConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_comparison_efficient_frontier.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&a, &b, &cfg, &path).unwrap();
    }
}
