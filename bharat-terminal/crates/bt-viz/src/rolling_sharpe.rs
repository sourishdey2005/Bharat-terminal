// crates/bt-viz/src/rolling_sharpe.rs
// Author: Sourish Dey

//! Rolling Sharpe ratio chart. Made by Sourish Dey.

use bt_analytics::rolling_sharpe;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct RollingSharpeConfig {
    pub title: String,
    pub theme: Theme,
    pub window: usize,
    pub risk_free: f64,
    pub periods_per_year: usize,
}

impl Default for RollingSharpeConfig {
    fn default() -> Self {
        Self {
            title: "Rolling Sharpe Ratio".to_string(),
            theme: Theme::Dark,
            window: 20,
            risk_free: 0.05,
            periods_per_year: 252,
        }
    }
}

impl RollingSharpeConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
    pub fn window(mut self, w: usize) -> Self { self.window = w; self }
    pub fn risk_free(mut self, r: f64) -> Self { self.risk_free = r; self }
    pub fn periods_per_year(mut self, p: usize) -> Self { self.periods_per_year = p; self }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &RollingSharpeConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let returns = series.returns();
    let rs = rolling_sharpe(&returns, cfg.window, cfg.risk_free, cfg.periods_per_year);
    let valid: Vec<f64> = rs.iter().filter(|v| v.is_finite()).cloned().collect();
    if valid.is_empty() {
        return Err(BtError::EmptySeries("rolling sharpe".into()));
    }
    let y_min = valid.iter().cloned().fold(f64::MAX, f64::min);
    let y_max = valid.iter().cloned().fold(f64::MIN, f64::max);
    let pad = (y_max - y_min).max(0.1) * 0.15;
    let n = rs.len() as f64;

    let mut chart = ChartBuilder::on(&root)
        .caption(&cfg.title, (TITLE_FONT, 22).into_font().color(&cfg.theme.text()))
        .margin(15)
        .x_label_area_size(35)
        .y_label_area_size(60)
        .build_cartesian_2d(0f64..n, (y_min - pad)..(y_max + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .x_desc("Bar")
        .y_desc("Sharpe Ratio")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let points: Vec<(f64, f64)> = rs
        .iter()
        .enumerate()
        .filter(|(_, v)| v.is_finite())
        .map(|(i, v)| (i as f64, *v))
        .collect();

    chart
        .draw_series(LineSeries::new(points.clone(), cfg.theme.accent().stroke_width(2)))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(0.0, 0.0), (n, 0.0)],
            cfg.theme.border().stroke_width(1),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &RollingSharpeConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &RollingSharpeConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = RollingSharpeConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_rolling_sharpe.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
