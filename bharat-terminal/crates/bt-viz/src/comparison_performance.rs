// crates/bt-viz/src/comparison_performance.rs
// Author: Sourish Dey

//! Performance comparison (normalized). Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct ComparisonPerformanceConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for ComparisonPerformanceConfig {
    fn default() -> Self {
        Self {
            title: "Performance Comparison".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl ComparisonPerformanceConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, title: impl Into<String>) -> Self { self.title = title.into(); self }
    pub fn theme(mut self, theme: Theme) -> Self { self.theme = theme; self }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series_a: &OhlcvSeries,
    series_b: &OhlcvSeries,
    cfg: &ComparisonPerformanceConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series_a.validate()?;
    series_b.validate()?;
    fill_background(&root, cfg.theme)?;

    let n = series_a.candles.len().min(series_b.candles.len());
    let base_a = series_a.candles[0].close;
    let base_b = series_b.candles[0].close;

    let norm_a: Vec<f64> = (0..n).map(|i| (series_a.candles[i].close / base_a - 1.0) * 100.0).collect();
    let norm_b: Vec<f64> = (0..n).map(|i| (series_b.candles[i].close / base_b - 1.0) * 100.0).collect();

    let t_min = 0.0;
    let t_max = n as f64;
    let y_min = norm_a.iter().chain(norm_b.iter()).copied().fold(0.0_f64, f64::min) * 1.1;
    let y_max = norm_a.iter().chain(norm_b.iter()).copied().fold(0.0_f64, f64::max) * 1.1;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} — {} vs {}", cfg.title, series_a.symbol, series_b.symbol),
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

    chart
        .draw_series(LineSeries::new(
            vec![(t_min, 0.0), (t_max, 0.0)],
            cfg.theme.border().mix(0.5).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(LineSeries::new(
            (0..n).map(|i| (i as f64, norm_a[i])),
            cfg.theme.profit().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label(&series_a.symbol)
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.profit().stroke_width(2)));

    chart
        .draw_series(LineSeries::new(
            (0..n).map(|i| (i as f64, norm_b[i])),
            cfg.theme.loss().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label(&series_b.symbol)
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.loss().stroke_width(2)));

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

pub fn render_png(series_a: &OhlcvSeries, series_b: &OhlcvSeries, cfg: &ComparisonPerformanceConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series_a, series_b, cfg)
}

pub fn render_svg(series_a: &OhlcvSeries, series_b: &OhlcvSeries, cfg: &ComparisonPerformanceConfig, path: &str) -> Result<()> {
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
        let cfg = ComparisonPerformanceConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_comparison_performance.png").to_str().unwrap().to_string();
        render_png(&a, &b, &cfg, &path).unwrap();
    }
}
