// crates/bt-viz/src/skew_evolution.rs
// Author: Sourish Dey

//! Skew evolution across expiries. Made by Sourish Dey.

use bt_core::{BtError, Result};
use plotters::prelude::*;

use crate::common::{draw_footer, fill_background, png_root, svg_root, LABEL_FONT, TITLE_FONT};
use crate::palette::Theme;

#[derive(Debug, Clone)]
pub struct SkewEvolutionConfig {
    pub title: String,
    pub theme: Theme,
    pub expiries: Vec<f64>,
    pub skew_values: Vec<f64>,
}

impl Default for SkewEvolutionConfig {
    fn default() -> Self {
        Self {
            title: "Skew Evolution".to_string(),
            theme: Theme::Dark,
            expiries: vec![],
            skew_values: vec![],
        }
    }
}

impl SkewEvolutionConfig {
    pub fn new() -> Self { Self::default() }
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = t.into(); self }
    pub fn theme(mut self, t: Theme) -> Self { self.theme = t; self }
    pub fn expiries(mut self, e: Vec<f64>) -> Self { self.expiries = e; self }
    pub fn skew_values(mut self, s: Vec<f64>) -> Self { self.skew_values = s; self }
}

fn render<DB: DrawingBackend>(
    root: plotters::drawing::DrawingArea<DB, plotters::coord::Shift>,
    cfg: &SkewEvolutionConfig,
) -> Result<()>
where DB::ErrorType: 'static,
{
    if cfg.expiries.is_empty() || cfg.skew_values.is_empty() {
        return Err(BtError::EmptySeries("skew data".into()));
    }
    fill_background(&root, cfg.theme)?;

    let x_min = cfg.expiries.iter().cloned().fold(f64::MAX, f64::min);
    let x_max = cfg.expiries.iter().cloned().fold(f64::MIN, f64::max);
    let y_min = cfg.skew_values.iter().cloned().fold(f64::MAX, f64::min);
    let y_max = cfg.skew_values.iter().cloned().fold(f64::MIN, f64::max);
    let pad = (y_max - y_min).max(0.1) * 0.15;

    let mut chart = ChartBuilder::on(&root)
        .caption(&cfg.title, (TITLE_FONT, 22).into_font().color(&cfg.theme.text()))
        .margin(15)
        .x_label_area_size(35)
        .y_label_area_size(60)
        .build_cartesian_2d(x_min..x_max, (y_min - pad)..(y_max + pad))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .configure_mesh()
        .label_style((LABEL_FONT, 12).into_font().color(&cfg.theme.text()))
        .axis_style(&cfg.theme.border())
        .x_desc("Expiry (years)")
        .y_desc("Skew")
        .draw()
        .map_err(|e| BtError::Render(e.to_string()))?;

    let points: Vec<(f64, f64)> = cfg
        .expiries
        .iter()
        .cloned()
        .zip(cfg.skew_values.iter().cloned())
        .collect();

    chart
        .draw_series(LineSeries::new(points.clone(), cfg.theme.accent().stroke_width(3)))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(points.iter().map(|&(x, y)| Circle::new((x, y), 5, cfg.theme.accent().filled())))
        .map_err(|e| BtError::Render(e.to_string()))?;

    chart
        .draw_series(std::iter::once(PathElement::new(
            vec![(x_min, 0.0), (x_max, 0.0)],
            cfg.theme.border().stroke_width(1),
        )))
        .map_err(|e| BtError::Render(e.to_string()))?;

    draw_footer(&root, cfg.theme)?;
    root.present().map_err(|e| BtError::Render(e.to_string()))?;
    Ok(())
}

pub fn render_png(cfg: &SkewEvolutionConfig, path: &str) -> Result<()> {
    render(png_root(path)?, cfg)
}

pub fn render_svg(cfg: &SkewEvolutionConfig, path: &str) -> Result<()> {
    render(svg_root(path)?, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders() {
        let cfg = SkewEvolutionConfig::new()
            .theme(Theme::Dark)
            .expiries(vec![0.1, 0.25, 0.5, 1.0, 2.0])
            .skew_values(vec![-0.3, -0.25, -0.2, -0.15, -0.1]);
        let path = std::env::temp_dir().join("bt_test_skew_evolution.png").to_str().unwrap().to_string();
        render_png(&cfg, &path).unwrap();
    }
}
