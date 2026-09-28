// crates/bt-viz/src/hurst.rs
// Author: Sourish Dey

//! Rolling Hurst exponent chart. Made by Sourish Dey.

use bt_core::{BtError, OhlcvSeries, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

/// Computes the Hurst exponent via rescaled range (R/S) analysis over
/// a rolling window. H < 0.5 = mean-reverting, H > 0.5 = trending.
pub fn hurst_exponent(series: &[f64]) -> f64 {
    let n = series.len();
    if n < 8 {
        return 0.5;
    }
    let mean = series.iter().sum::<f64>() / n as f64;
    let mut cum = vec![0.0_f64; n];
    let mut acc = 0.0;
    for (i, &v) in series.iter().enumerate() {
        acc += v - mean;
        cum[i] = acc;
    }
    let r = cum.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
        - cum.iter().cloned().fold(f64::INFINITY, f64::min);
    let std = (series.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64).sqrt();
    if std <= 0.0 || r <= 0.0 {
        return 0.5;
    }
    let rs = r / std;
    (rs.ln() / (n as f64).ln()).clamp(0.05, 0.95)
}

/// Rolling Hurst exponent over `window` points.
pub fn rolling_hurst(returns: &[f64], window: usize) -> Vec<f64> {
    let mut out = vec![f64::NAN; returns.len()];
    if window == 0 || returns.len() < window {
        return out;
    }
    for i in (window - 1)..returns.len() {
        out[i] = hurst_exponent(&returns[i + 1 - window..=i]);
    }
    out
}

#[derive(Debug, Clone)]
pub struct HurstConfig {
    pub title: String,
    pub theme: Theme,
    pub window: usize,
}

impl Default for HurstConfig {
    fn default() -> Self {
        Self {
            title: "Hurst Exponent".to_string(),
            theme: Theme::Dark,
            window: 30,
        }
    }
}

impl HurstConfig {
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
        self.window = w.max(8);
        self
    }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    series: &OhlcvSeries,
    cfg: &HurstConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    series.validate()?;
    let returns = series.returns();
    if returns.len() < cfg.window + 1 {
        return Err(BtError::InvalidInput(format!(
            "series too short ({}) for hurst window {}",
            returns.len(),
            cfg.window
        )));
    }
    fill_background(&root, cfg.theme)?;

    let hurst = rolling_hurst(&returns, cfg.window);
    let valid: Vec<f64> = hurst.iter().copied().filter(|v| !v.is_nan()).collect();
    if valid.is_empty() {
        return Err(BtError::InvalidInput("no valid hurst values".into()));
    }
    let h_min = valid.iter().cloned().fold(f64::INFINITY, f64::min);
    let h_max = valid.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let y_lo = (h_min - 0.05).max(0.0);
    let y_hi = (h_max + 0.05).min(1.0);

    let mut chart = ChartBuilder::on(&root)
        .caption(
            format!("{} (window={})", cfg.title, cfg.window),
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(15)
        .x_label_area_size(30)
        .y_label_area_size(50)
        .build_cartesian_2d(0f64..hurst.len() as f64, y_lo..y_hi)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 11).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    // Random-walk reference line
    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(0.0, 0.5), (hurst.len() as f64, 0.5)],
            cfg.theme.info().stroke_width(1),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    // H > 0.5 zone (trending)
    chart
        .draw_series(std::iter::once(Rectangle::new(
            [(0.0, 0.5), (hurst.len() as f64, y_hi)],
            cfg.theme.profit().mix(0.06).filled(),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    // H < 0.5 zone (mean-reverting)
    chart
        .draw_series(std::iter::once(Rectangle::new(
            [(0.0, y_lo), (hurst.len() as f64, 0.5)],
            cfg.theme.loss().mix(0.06).filled(),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    let pts: Vec<(f64, f64)> = hurst
        .iter()
        .enumerate()
        .filter(|(_, v)| !v.is_nan())
        .map(|(i, &v)| (i as f64, v))
        .collect();

    chart
        .draw_series(LineSeries::new(pts, cfg.theme.accent().stroke_width(2)))
        .map_err(|e| BtError::Render(e.to_string()))?;

    // Current value annotation
    if let Some(&last) = valid.last() {
        root.draw(&Text::new(
            format!("H = {:.3}", last),
            (50, 45),
            (LABEL_FONT, 14).into_font().color(&cfg.theme.accent()),
        ))
        .map_err(|e| BtError::Render(e.to_string()))?;
    }

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(series: &OhlcvSeries, cfg: &HurstConfig, path: &str) -> Result<()> {
    render(png_root(path)?, series, cfg)
}

pub fn render_svg(series: &OhlcvSeries, cfg: &HurstConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, series, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bt_core::synthetic_ohlcv;

    #[test]
    fn renders() {
        let series = synthetic_ohlcv("TEST", 200, 1, 100.0);
        let cfg = HurstConfig::new().theme(Theme::Dark);
        let path = std::env::temp_dir()
            .join("bt_test_hurst.png")
            .to_str()
            .unwrap()
            .to_string();
        render_png(&series, &cfg, &path).unwrap();
    }
}
