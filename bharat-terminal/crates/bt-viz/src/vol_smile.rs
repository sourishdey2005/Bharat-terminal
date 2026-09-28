//! Tier 2 #10 â€” Volatility smile/skew, multi-expiry. Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

/// One expiry's implied-vol smile: strikes (as % moneyness) -> IV.
#[derive(Debug, Clone)]
pub struct SmileCurve {
    pub label: String,
    pub moneyness: Vec<f64>,
    pub iv: Vec<f64>,
}

impl SmileCurve {
    pub fn new(label: impl Into<String>, moneyness: Vec<f64>, iv: Vec<f64>) -> Self {
        Self {
            label: label.into(),
            moneyness,
            iv,
        }
    }
}

/// Builds a synthetic smile: a parabola in log-moneyness plus per-expiry
/// skew, useful for demos/tests without any options-data provider.
pub fn synthetic_smile(label: &str, atm_iv: f64, skew: f64, n_strikes: usize) -> SmileCurve {
    let mut moneyness = Vec::with_capacity(n_strikes);
    let mut iv = Vec::with_capacity(n_strikes);
    for i in 0..n_strikes {
        let m = -0.3 + 0.6 * (i as f64) / (n_strikes.max(1) - 1).max(1) as f64;
        let curvature = 0.35 * m * m;
        let v = (atm_iv + curvature - skew * m).max(0.02);
        moneyness.push(m);
        iv.push(v);
    }
    SmileCurve::new(label, moneyness, iv)
}

#[derive(Debug, Clone)]
pub struct VolSmileConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for VolSmileConfig {
    fn default() -> Self {
        Self {
            title: "Volatility Smile / Skew".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl VolSmileConfig {
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
    curves: &[SmileCurve],
    cfg: &VolSmileConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if curves.is_empty() {
        return Err(BtError::EmptySeries("volatility smile curves".into()));
    }
    fill_background(&root, cfg.theme)?;

    let x_min = curves
        .iter()
        .flat_map(|c| c.moneyness.iter().cloned())
        .fold(f64::MAX, f64::min);
    let x_max = curves
        .iter()
        .flat_map(|c| c.moneyness.iter().cloned())
        .fold(f64::MIN, f64::max);
    let y_min = curves
        .iter()
        .flat_map(|c| c.iv.iter().cloned())
        .fold(f64::MAX, f64::min);
    let y_max = curves
        .iter()
        .flat_map(|c| c.iv.iter().cloned())
        .fold(f64::MIN, f64::max);
    let pad = (y_max - y_min).max(0.01) * 0.15;

    let mut chart = ChartBuilder::on(&root)
        .caption(
            &cfg.title,
            (TITLE_FONT, 22).into_font().color(&cfg.theme.text()),
        )
        .margin(15)
        .x_label_area_size(35)
        .y_label_area_size(60)
        .build_cartesian_2d(x_min..x_max, (y_min - pad)..(y_max + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .x_desc("Log-moneyness")
        .y_desc("Implied Vol")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let palette = cfg.theme.categorical(0);
    for (idx, curve) in curves.iter().enumerate() {
        let color = palette[idx % palette.len()];
        let points: Vec<(f64, f64)> = curve
            .moneyness
            .iter()
            .cloned()
            .zip(curve.iv.iter().cloned())
            .collect();
        chart
            .draw_series(LineSeries::new(points.clone(), color.stroke_width(2)))
            .map_err(|e| BtError::Render(e.to_string()))?
            .label(curve.label.clone())
            .legend(move |(x, y)| {
                PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2))
            });

        chart
            .draw_series(
                points
                    .iter()
                    .map(|&(x, y)| Circle::new((x, y), 3, color.filled())),
            )
            .map_err(|e| BtError::Render(e.to_string()))?;
    }

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

pub fn render_png(curves: &[SmileCurve], cfg: &VolSmileConfig, path: &str) -> Result<()> {
    render(png_root(path)?, curves, cfg)
}

pub fn render_svg(curves: &[SmileCurve], cfg: &VolSmileConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, curves, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_constructs() {
        let cfg = VolSmileConfig::new();
        let _ = cfg;
    }
}
