//! Tier 3 #16 â€” Drawdown underwater chart. Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct DrawdownConfig {
    pub title: String,
    pub theme: Theme,
}

impl Default for DrawdownConfig {
    fn default() -> Self {
        Self {
            title: "Drawdown â€” Underwater Chart".to_string(),
            theme: Theme::Dark,
        }
    }
}

impl DrawdownConfig {
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

/// Computes the running drawdown (%) series from a price/equity curve.
pub fn compute_drawdown(equity: &[f64]) -> Vec<f64> {
    let mut peak = f64::MIN;
    equity
        .iter()
        .map(|&v| {
            peak = peak.max(v);
            if peak <= 0.0 {
                0.0
            } else {
                (v - peak) / peak * 100.0
            }
        })
        .collect()
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    equity: &[f64],
    cfg: &DrawdownConfig,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    if equity.is_empty() {
        return Err(BtError::EmptySeries("equity curve".into()));
    }
    fill_background(&root, cfg.theme)?;

    let dd = compute_drawdown(equity);
    let min_dd = dd.iter().cloned().fold(0.0_f64, f64::min);
    let n = dd.len() as f64;

    let mut chart = ChartBuilder::on(&root)
        .caption(&cfg.title, (TITLE_FONT, 22).into_font().color(&cfg.theme.text()))
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(60)
        .build_cartesian_2d(0f64..n, (min_dd * 1.1).min(-1.0)..0.5f64)
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .y_desc("Drawdown (%)")
        .x_desc("Bar")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(AreaSeries::new(
            dd.iter().enumerate().map(|(i, &v)| (i as f64, v)),
            0.0,
            cfg.theme.loss().mix(0.35),
        ).border_style(cfg.theme.loss().stroke_width(2)))
        .map_err(|e| BtError::Render(e.to_string()))?;

    // Mark recovery points: where drawdown returns to (near) zero after
    // having been negative.
    let mut recovery_points = Vec::new();
    let mut was_under = false;
    for (i, &v) in dd.iter().enumerate() {
        if v < -0.01 {
            was_under = true;
        } else if was_under {
            recovery_points.push((i as f64, v));
            was_under = false;
        }
    }
    chart
        .draw_series(
            recovery_points
                .iter()
                .map(|&(x, y)| Circle::new((x, y), 4, cfg.theme.accent().filled())),
        )
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(equity: &[f64], cfg: &DrawdownConfig, path: &str) -> Result<()> {
    render(png_root(path)?, equity, cfg)
}

pub fn render_svg(equity: &[f64], cfg: &DrawdownConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, equity, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_constructs() {
        let cfg = DrawdownConfig::new();
        let _ = cfg;
    }
}
