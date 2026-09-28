// crates/bt-viz/src/rolling_correlation.rs
// Author: Sourish Dey

//! Rolling correlation chart. Made by Sourish Dey.

use bt_analytics::rolling_correlation;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct RollingCorrelationConfig {
    pub title: String,
    pub theme: Theme,
    pub window: usize,
    pub benchmark: OhlcvSeries,
}

impl Default for RollingCorrelationConfig {
    fn default() -> Self {
        Self {
            title: "Rolling Correlation".to_string(),
            theme: Theme::Dark,
            window: 20,
            benchmark: OhlcvSeries::new("BENCH", vec![]),
        }
    }
}

impl RollingCorrelationConfig {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = t.into();
        self
    }
    pub fn theme(mut self, t: Theme) -> Self {
        self.theme = t;
        self
    }
    pub fn window(mut self, w: usize) -> Self {
        self.window = w;
        self
    }
    pub fn benchmark(mut self, b: OhlcvSeries) -> Self {
        self.benchmark = b;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &RollingCorrelationConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let returns_a = series.returns();
    let returns_b = cfg.benchmark.returns();

    if returns_a.len() != returns_b.len() || returns_a.is_empty() {
        return Err(BtError::InvalidInput("series length mismatch".into()));
    }

    let rc = rolling_correlation(&returns_a, &returns_b, cfg.window);
    let n = rc.len() as f64;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            &cfg.title,
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(15)
        .x_label_area_size(35)
        .y_label_area_size(60)
        .build_cartesian_2d(0f64..n, -1.0..1.0)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .x_desc("Bar")
        .y_desc("Correlation")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let points: Vec<(f64, f64)> = rc
        .iter()
        .enumerate()
        .filter(|(_, v)| v.is_finite())
        .map(|(i, v)| (i as f64, *v))
        .collect();

    chart
        .draw_series(LineSeries::new(
            points.clone(),
            cfg.theme.accent().stroke_width(2),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(0.0, 0.0), (n, 0.0)],
            cfg.theme.border().stroke_width(1),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(0.0, 1.0), (n, 1.0)],
            cfg.theme.border().mix(0.5).stroke_width(1),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(0.0, -1.0), (n, -1.0)],
            cfg.theme.border().mix(0.5).stroke_width(1),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &RollingCorrelationConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &RollingCorrelationConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let bench = synthetic_ohlcv("BENCH", 100, 2, 100.0);
        let cfg = RollingCorrelationConfig::new()
            .theme(Theme::Dark)
            .benchmark(bench);
        let path = std::env::temp_dir()
            .join("bt_test_rolling_correlation.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
