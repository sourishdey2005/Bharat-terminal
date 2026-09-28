//! Tier 6 #43 â€” Yield Curve Family (multi-day overlaid curves). Made by
//! Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

/// A single day's yield curve: tenor (years) -> yield (%).
#[derive(Debug, Clone)]
pub struct YieldCurve {
    pub date_label: String,
    pub tenors: Vec<f64>,
    pub yields: Vec<f64>,
}

impl YieldCurve {
    pub fn new(date_label: impl Into<String>, tenors: Vec<f64>, yields: Vec<f64>) -> Self {
        Self {
            date_label: date_label.into(),
            tenors,
            yields,
        }
    }
}

/// Builds a synthetic Nelson-Siegel-style curve for demos/tests: level,
/// slope and curvature parameters map directly to short/long-end shape.
pub fn synthetic_curve(
    date_label: &str,
    level: f64,
    slope: f64,
    curvature: f64,
    tenors: &[f64],
) -> YieldCurve {
    let lambda = 1.5;
    let ys: Vec<f64> = tenors
        .iter()
        .map(|&t| {
            let x = (t / lambda).max(1e-6);
            let decay = (-x).exp();
            let slope_term = (1.0 - decay) / x;
            let curve_term = slope_term - decay;
            (level + slope * slope_term + curvature * curve_term).max(0.0)
        })
        .collect();
    YieldCurve::new(date_label, tenors.to_vec(), ys)
}

#[derive(Debug, Clone)]
pub struct YieldCurveConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for YieldCurveConfig {
    fn default() -> Self {
        Self {
            title: "Yield Curve Family".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl YieldCurveConfig {
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
    curves: &[YieldCurve],
    cfg: &YieldCurveConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if curves.is_empty() {
        return Err(BtError::EmptySeries("yield curves".into()));
    }
    fill_background(&root, cfg.theme)?;

    let x_max = curves
        .iter()
        .flat_map(|c| c.tenors.iter().cloned())
        .fold(0.0_f64, f64::max);
    let y_min = curves
        .iter()
        .flat_map(|c| c.yields.iter().cloned())
        .fold(f64::MAX, f64::min);
    let y_max = curves
        .iter()
        .flat_map(|c| c.yields.iter().cloned())
        .fold(f64::MIN, f64::max);
    let pad = (y_max - y_min).max(0.1) * 0.15;

    let mut chart = ChartBuilder::on(&root)
        .caption(&cfg.title, (TITLE_FONT, 22).into_font().color(&cfg.theme.text()))
        .margin(15)
        .x_label_area_size(35)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0..x_max * 1.05, (y_min - pad)..(y_max + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .x_desc("Tenor (years)")
        .y_desc("Yield (%)")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let palette = cfg.theme.categorical(20);
    for (idx, curve) in curves.iter().enumerate() {
        let color = palette[idx % palette.len()];
        let alpha = 0.4 + 0.6 * (idx as f64 + 1.0) / curves.len() as f64;
        let points: Vec<(f64, f64)> = curve.tenors.iter().cloned().zip(curve.yields.iter().cloned()).collect();
        chart
            .draw_series(LineSeries::new(points.clone(), color.mix(alpha).stroke_width(2)))
            .map_err(|e| BtError::Render(e.to_string()))?
            .label(curve.date_label.clone())
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2)));
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

pub fn render_png(curves: &[YieldCurve], cfg: &YieldCurveConfig, path: &str) -> Result<()> {
    render(png_root(path)?, curves, cfg)
}

pub fn render_svg(curves: &[YieldCurve], cfg: &YieldCurveConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, curves, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_constructs() {
        let cfg = YieldCurveConfig::new();
        let _ = cfg;
    }
}
