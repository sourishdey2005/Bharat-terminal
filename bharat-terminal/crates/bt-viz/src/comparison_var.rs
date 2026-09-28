// crates/bt-viz/src/comparison_var.rs
// Author: Sourish Dey

//! VaR comparison across assets. Made by Sourish Dey.

use bt_analytics::var_historical;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct ComparisonVarConfig {
    pub title: String,
    pub theme: Theme,
    pub confidence: f64,
}

impl Default for ComparisonVarConfig {
    fn default() -> Self {
        Self {
            title: "VaR Comparison".to_string(),
            theme: Theme::Dark,
            confidence: 0.95,
        }
    }
}

impl ComparisonVarConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, title: impl Into<String>) -> Self { self.title = title.into(); self }
    pub fn theme(mut self, theme: Theme) -> Self { self.theme = theme; self }
    pub fn confidence(mut self, v: f64) -> Self { self.confidence = v.clamp(0.5, 0.999); self }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series_a: &OhlcvSeries,
    series_b: &OhlcvSeries,
    cfg: &ComparisonVarConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series_a.validate()?;
    series_b.validate()?;
    fill_background(&root, cfg.theme)?;

    let returns_a = series_a.returns();
    let returns_b = series_b.returns();
    let var_a = var_historical(&returns_a, cfg.confidence);
    let var_b = var_historical(&returns_b, cfg.confidence);

    let max_var = var_a.max(var_b) * 1.2;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — VaR({:.1}%): {}={:.2}%, {}={:.2}%",
                cfg.title, cfg.confidence, series_a.symbol, var_a, series_b.symbol, var_b),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..2.0, 0.0..max_var)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .light_line_style(cfg.theme.border().mix(0.3))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(vec![
            Rectangle::new([(0.2, 0.0), (0.8, var_a)], cfg.theme.profit().filled()),
            Rectangle::new([(1.2, 0.0), (1.8, var_b)], cfg.theme.loss().filled()),
        ])
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series_a: &OhlcvSeries, series_b: &OhlcvSeries, cfg: &ComparisonVarConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series_a, series_b, cfg)
}

pub fn render_svg(series_a: &OhlcvSeries, series_b: &OhlcvSeries, cfg: &ComparisonVarConfig, path: &str) -> Result<()> {
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
        let cfg = ComparisonVarConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_comparison_var.png").to_str().unwrap().to_string();
        render_png(&a, &b, &cfg, &path).unwrap();
    }
}
