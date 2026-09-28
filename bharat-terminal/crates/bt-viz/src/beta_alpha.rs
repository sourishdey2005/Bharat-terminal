// crates/bt-viz/src/beta_alpha.rs
// Author: Sourish Dey

//! Beta/Alpha scatter plot. Made by Sourish Dey.

use bt_analytics::{alpha, beta};
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct BetaAlphaConfig {
    pub title: String,
    pub theme: Theme,
    pub benchmark: OhlcvSeries,
}

impl Default for BetaAlphaConfig {
    fn default() -> Self {
        Self {
            title: "Beta / Alpha Scatter".to_string(),
            theme: Theme::Dark,
            benchmark: OhlcvSeries::new("BENCH", vec![]),
        }
    }
}

impl BetaAlphaConfig {
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
    pub fn benchmark(mut self, b: OhlcvSeries) -> Self {
        self.benchmark = b;
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &BetaAlphaConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let asset_returns = series.returns();
    let bench_returns = cfg.benchmark.returns();

    if asset_returns.len() != bench_returns.len() || asset_returns.is_empty() {
        return Err(BtError::InvalidInput("series length mismatch".into()));
    }

    let b = beta(&asset_returns, &bench_returns);
    let a = alpha(&asset_returns, &bench_returns, 0.05, 252);

    let mut chart = ChartBuilder::on(&root)
        .caption(
            &cfg.title,
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(15)
        .x_label_area_size(35)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..2.0, -0.02..0.02)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .x_desc("Beta")
        .y_desc("Alpha (Jensen's)")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(std::iter::once(Circle::new(
            (b, a),
            8,
            cfg.theme.accent().filled(),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(std::iter::once(Text::new(
            format!("{}\nBeta: {:.3}\nAlpha: {:.4}", series.symbol, b, a),
            (b + 0.05, a + 0.002),
            (LABEL_FONT, 13).into_font().color(&cfg.theme.text()),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(0.0, 0.0), (2.0, 0.0)],
            cfg.theme.border().stroke_width(1),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &BetaAlphaConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &BetaAlphaConfig, path: &str) -> Result<()> {
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
        let cfg = BetaAlphaConfig::new().theme(Theme::Dark).benchmark(bench);
        let path = std::env::temp_dir()
            .join("bt_test_beta_alpha.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
