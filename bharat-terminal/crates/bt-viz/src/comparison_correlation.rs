// crates/bt-viz/src/comparison_correlation.rs
// Author: Sourish Dey

//! Correlation comparison across symbols. Made by Sourish Dey.

use bt_analytics::correlation;
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct ComparisonCorrelationConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for ComparisonCorrelationConfig {
    fn default() -> Self {
        Self {
            title: "Correlation Comparison".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl ComparisonCorrelationConfig {
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
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series_a: &OhlcvSeries,
    series_b: &OhlcvSeries,
    cfg: &ComparisonCorrelationConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series_a.validate()?;
    series_b.validate()?;
    fill_background(&root, cfg.theme)?;

    let returns_a = series_a.returns();
    let returns_b = series_b.returns();
    let corr = correlation(&returns_a, &returns_b);

    let n = returns_a.len().min(returns_b.len());
    let t_min = 0.0;
    let t_max = n as f64;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!(
                "{} — {} vs {} (r={:.3})",
                cfg.title, series_a.symbol, series_b.symbol, corr
            ),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(t_min..t_max, -1.0..1.0)
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

    let points: Vec<(f64, f64)> = (0..n)
        .map(|i| (i as f64, returns_a[i] * returns_b[i] * 100.0))
        .collect();

    chart
        .draw_series(
            points
                .iter()
                .map(|(x, y)| Circle::new((*x, *y), 2, cfg.theme.info().mix(0.5).filled())),
        )
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(LineSeries::new(
            points.iter().map(|(x, y)| (*x, *y)),
            cfg.theme.accent().mix(0.3).stroke_width(1),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(
    series_a: &OhlcvSeries,
    series_b: &OhlcvSeries,
    cfg: &ComparisonCorrelationConfig,
    path: &str,
) -> Result<()> {
    render(png_root(path)?, series_a, series_b, cfg)
}

pub fn render_svg(
    series_a: &OhlcvSeries,
    series_b: &OhlcvSeries,
    cfg: &ComparisonCorrelationConfig,
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
        let cfg = ComparisonCorrelationConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_comparison_correlation.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&a, &b, &cfg, &path).unwrap();
    }
}
