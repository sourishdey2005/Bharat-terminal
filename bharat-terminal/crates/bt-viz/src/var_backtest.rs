// crates/bt-viz/src/var_backtest.rs
// Author: Sourish Dey

//! VaR backtest fan chart. Made by Sourish Dey.

use bt_analytics::{cvar, var_historical};
use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct VarBacktestConfig {
    pub title: String,
    pub theme: Theme,
    pub confidence: f64,
    pub window: usize,
}

impl Default for VarBacktestConfig {
    fn default() -> Self {
        Self {
            title: "VaR Backtest Fan".to_string(),
            theme: Theme::Dark,
            confidence: 0.95,
            window: 20,
        }
    }
}

impl VarBacktestConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
    pub fn confidence(mut self, c: f64) -> Self { self.confidence = c; self }
    pub fn window(mut self, w: usize) -> Self { self.window = w; self }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &VarBacktestConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    series.validate()?;
    fill_background(&root, cfg.theme)?;

    let returns = series.returns();
    if returns.len() < cfg.window {
        return Err(BtError::InvalidInput("insufficient data for VaR window".into()));
    }

    let mut var_series = Vec::with_capacity(returns.len());
    let mut cvar_series = Vec::with_capacity(returns.len());
    for i in 0..returns.len() {
        if i + 1 >= cfg.window {
            let window = &returns[i + 1 - cfg.window..=i];
            var_series.push(var_historical(window, cfg.confidence));
            cvar_series.push(cvar(window, cfg.confidence));
        } else {
            var_series.push(0.0);
            cvar_series.push(0.0);
        }
    }

    let y_max = cvar_series
        .iter()
        .cloned()
        .fold(0.0_f64, f64::max)
        .max(0.1);
    let n = returns.len() as f64;

    let mut chart = ChartBuilder::on(&root)
        .caption(&cfg.title, (TITLE_FONT, 22).into_font().color(&cfg.theme.text()))
        .margin(15)
        .x_label_area_size(35)
        .y_label_area_size(60)
        .build_cartesian_2d(0f64..n, 0.0..(y_max * 1.2))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .x_desc("Bar")
        .y_desc("Loss (%)")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let var_points: Vec<(f64, f64)> = var_series
        .iter()
        .enumerate()
        .map(|(i, v)| (i as f64, *v))
        .collect();
    let cvar_points: Vec<(f64, f64)> = cvar_series
        .iter()
        .enumerate()
        .map(|(i, v)| (i as f64, *v))
        .collect();

    chart
        .draw_series(AreaSeries::new(
            var_points.clone(),
            0.0,
            cfg.theme.info().mix(0.25),
        ).border_style(cfg.theme.info().stroke_width(2)))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("VaR")
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.info().stroke_width(2)));

    chart
        .draw_series(AreaSeries::new(
            cvar_points.clone(),
            0.0,
            cfg.theme.loss().mix(0.25),
        ).border_style(cfg.theme.loss().stroke_width(2)))
        .map_err(|e| BtError::Render(e.to_string()))?
        .label("CVaR")
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], cfg.theme.loss().stroke_width(2)));

    chart
        .configure_series_labels()
        .background_style(cfg.theme.background().mix(0.8))
        .border_style(cfg.theme.border())
        .label_font((LABEL_FONT, 13).into_font().color(&cfg.theme.text()))
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &VarBacktestConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &VarBacktestConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 100, 1, 100.0);
        let cfg = VarBacktestConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir().join("bt_test_var_backtest.png").to_str().unwrap().to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
