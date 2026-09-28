// crates/bt-viz/src/comparison_sharpe.rs
// Author: Sourish Dey

//! Sharpe ratio comparison. Made by Sourish Dey.

use bt_analytics::sharpe;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct ComparisonSharpeConfig {
    pub title: String,
    pub theme: Theme,
    pub risk_free: f64,
    pub periods_per_year: usize,
}

impl Default for ComparisonSharpeConfig {
    fn default() -> Self {
        Self {
            title: "Sharpe Ratio Comparison".to_string(),
            theme: Theme::Dark,
            risk_free: 0.05,
            periods_per_year: 252,
        }
    }
}

impl ComparisonSharpeConfig {
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
    pub fn risk_free(mut self, v: f64) -> Self {
        self.risk_free = v;
        self
    }
    pub fn periods_per_year(mut self, v: usize) -> Self {
        self.periods_per_year = v.max(1);
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series_a: &OhlcvSeries,
    series_b: &OhlcvSeries,
    cfg: &ComparisonSharpeConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series_a.validate()?;
    series_b.validate()?;
    fill_background(&root, cfg.theme)?;

    let returns_a = series_a.returns();
    let returns_b = series_b.returns();
    let sharpe_a = sharpe(&returns_a, cfg.risk_free, cfg.periods_per_year);
    let sharpe_b = sharpe(&returns_b, cfg.risk_free, cfg.periods_per_year);

    let max_sharpe = sharpe_a.max(sharpe_b).abs().max(1.0) * 1.2;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!(
                "{} — Sharpe: {}={:.2}, {}={:.2}",
                cfg.title, series_a.symbol, sharpe_a, series_b.symbol, sharpe_b
            ),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..2.0, -max_sharpe..max_sharpe)
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
            vec![(0.0, 0.0), (2.0, 0.0)],
            cfg.theme.border().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(vec![
            Rectangle::new([(0.2, 0.0), (0.8, sharpe_a)], cfg.theme.profit().filled()),
            Rectangle::new([(1.2, 0.0), (1.8, sharpe_b)], cfg.theme.info().filled()),
        ])
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(
    series_a: &OhlcvSeries,
    series_b: &OhlcvSeries,
    cfg: &ComparisonSharpeConfig,
    path: &str,
) -> Result<()> {
    render(png_root(path)?, series_a, series_b, cfg)
}

pub fn render_svg(
    series_a: &OhlcvSeries,
    series_b: &OhlcvSeries,
    cfg: &ComparisonSharpeConfig,
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
        let cfg = ComparisonSharpeConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_comparison_sharpe.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&a, &b, &cfg, &path).unwrap();
    }
}
